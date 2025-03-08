use std::fmt;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::state::AppState;
use common::{
    Chats, Error, Event, Friend, FriendShip, Hub, HungUpReson, IceCandidate, Message, Result, Room,
    Store,
};

/// A Client with a connection of user websocket
#[derive(Clone)]
pub struct Client {
    id: Uuid,
    user_id: i64,
    store: Store,
    hub: Hub,
}

impl Client {
    pub fn new(user_id: i64, state: AppState) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            store: state.store,
            hub: state.hub,
        }
    }

    /// register a connection in the Hub
    pub async fn register(&self, tx: broadcast::Sender<Vec<u8>>) -> Result<()> {
        let Chats {
            rooms,
            friends,
            messages_map,
        } = Chats::init(self.user_id, &self.store).await?;

        self.hub.register(self.user_id, self.id, &rooms, tx.clone());

        tx.send(serde_json::to_vec(&Event::InitRooms(rooms))?)?;
        tx.send(serde_json::to_vec(&Event::InitFriends(friends))?)?;
        tx.send(serde_json::to_vec(&Event::InitMessages(messages_map))?)?;

        Ok(())
    }

    /// unregister the connection to Hub
    pub fn unregister(&self) {
        self.hub.unregister(self.user_id, &self.id);
    }

    /// process Event from user
    pub async fn process(&self, event: Event) -> Result<()> {
        let ret = match event {
            Event::Send(message) => self.send_message(message).await,
            Event::AddFriend(friend_id) => self.add_friend(friend_id).await,
            Event::AcceptFriend(friend_id) => self.accept_friend(friend_id).await,
            Event::RevertFriend(friend_id) => self.revert_friend(friend_id).await,
            Event::DeleteFriend(friend_id) => self.delete_friend(friend_id).await,
            Event::SendCall(friend_id) => self.call(friend_id),
            Event::SendHungUp(friend_id, reson) => self.hung_up(friend_id, reson),
            Event::SendReply(friend_id, client_id) => self.reply(friend_id, client_id),
            Event::SendOffer(friend_id, client_id, offer) => {
                self.send_offer(friend_id, client_id, offer)
            }
            Event::SendAnswer(friend_id, client_id, answer) => {
                self.send_answer(friend_id, client_id, answer)
            }
            Event::SendCandidate(friend_id, client_id, candidate) => {
                self.send_candidate(friend_id, client_id, candidate)
            }
            _ => Ok(()),
        };
        if let Err(err) = ret {
            match err {
                Error::SendError => Err(Error::SendError),
                _ => {
                    log::error!("event process error: {}", err);
                    Ok(())
                }
            }
        } else {
            ret
        }
    }

    async fn send_message(&self, message: Message) -> Result<()> {
        let message = self.hub.broadcast(message)?;
        message.cache(&self.store).await?;

        Ok(())
    }

    async fn add_friend(&self, friend_id: i64) -> Result<()> {
        let fsp = FriendShip::add(self.user_id, friend_id, &self.store).await?;
        let (user, friend) = Friend::get(self.user_id, &fsp, &self.store).await?;

        self.hub.send(self.user_id, &Event::ReceiveFriend(friend))?;
        self.hub.send(friend_id, &Event::ReceiveFriend(user))?;

        Ok(())
    }

    async fn accept_friend(&self, friend_id: i64) -> Result<()> {
        let fsp = FriendShip::accept(self.user_id, friend_id, &self.store).await?;
        let (user, friend) = Room::get(self.user_id, &fsp, &self.store).await?;

        self.hub.create_friend_room(fsp);
        self.hub.send(self.user_id, &Event::ReceiveRoom(friend))?;
        self.hub.send(friend_id, &Event::ReceiveRoom(user))?;

        Ok(())
    }

    async fn revert_friend(&self, friend_id: i64) -> Result<()> {
        FriendShip::revert(self.user_id, friend_id, &self.store).await?;

        let event = Event::RevertFriend(friend_id);
        self.hub.send(self.user_id, &event)?;

        let event = Event::RevertFriend(self.user_id);
        self.hub.send(friend_id, &event)?;

        Ok(())
    }

    async fn delete_friend(&self, friend_id: i64) -> Result<()> {
        let fsp = FriendShip::delete(self.user_id, friend_id, &self.store).await?;
        self.hub.remove_friend_room(fsp);

        let event = Event::DeleteFriend(friend_id);
        self.hub.send(self.user_id, &event)?;

        let event = Event::DeleteFriend(self.user_id);
        self.hub.send(friend_id, &event)?;

        Ok(())
    }

    fn call(&self, friend_id: i64) -> Result<()> {
        let reson = self.hub.make_call(self.user_id, friend_id)?;
        match reson {
            HungUpReson::Busy | HungUpReson::Offline => {
                let event = Event::ReceiveHungUp(reson);
                self.hub.notify(self.user_id, &self.id, event)?;
            }
            _ => {
                let event = Event::SendCallDone(friend_id);
                self.hub.notify(self.user_id, &self.id, event)?;

                let event = Event::ReceiveCall(self.user_id, self.id);
                self.hub.send(friend_id, &event)?;
            }
        };
        Ok(())
    }

    fn hung_up(&self, friend_id: i64, reson: HungUpReson) -> Result<()> {
        self.hub.make_hung_up(self.user_id, friend_id)?;

        let event = Event::ReceiveHungUp(reson);
        self.hub.send(self.user_id, &event)?;
        self.hub.send(friend_id, &event)?;
        Ok(())
    }

    fn reply(&self, friend_id: i64, client_id: Uuid) -> Result<()> {
        let success = self
            .hub
            .notify(friend_id, &client_id, Event::ReceiveReply(self.id))?;

        if !success {
            self.target_offline()?;
        }
        Ok(())
    }

    fn send_offer(&self, friend_id: i64, client_id: Uuid, offer: String) -> Result<()> {
        self.hub
            .notify(friend_id, &client_id, Event::ReceiveOffer(offer))?;

        Ok(())
    }

    fn send_answer(&self, friend_id: i64, client_id: Uuid, answer: String) -> Result<()> {
        self.hub
            .notify(friend_id, &client_id, Event::ReceiveAnswer(answer))?;

        Ok(())
    }

    fn send_candidate(
        &self,
        friend_id: i64,
        client_id: Uuid,
        candidate: IceCandidate,
    ) -> Result<()> {
        self.hub
            .notify(friend_id, &client_id, Event::ReceiveCandidate(candidate))?;

        Ok(())
    }

    fn target_offline(&self) -> Result<()> {
        self.hub.notify(
            self.user_id,
            &self.id,
            Event::ReceiveHungUp(HungUpReson::Offline),
        )?;
        Ok(())
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.user_id, self.id)
    }
}
