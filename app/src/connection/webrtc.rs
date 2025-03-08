use leptos::*;
use uuid::Uuid;

use super::WebSocketState;
use crate::components::Toast;
use common::{Event, HungUpReson};

cfg_if::cfg_if! { if #[cfg(feature = "hydrate")] {
    use wasm_bindgen::{prelude::*, UnwrapThrowExt};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::js_sys::Reflect;
    use web_sys::{
        HtmlAudioElement, MediaStream, MediaStreamConstraints, MediaStreamTrack,
        RtcIceCandidate, RtcIceCandidateInit, RtcPeerConnectionIceEvent, RtcSdpType,
        RtcSessionDescriptionInit, RtcTrackEvent, RtcOfferOptions, RtcPeerConnection,
    };
    use common::IceCandidate;
}}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", allow(dead_code))]
#[repr(u8)]
pub enum RtcStatus {
    Caller,
    Callee,
    Calling,
    Idle,
}

#[derive(Clone)]
pub struct WebRtcState(StoredValue<WebRtcInner>);

impl Copy for WebRtcState {}

#[derive(Clone)]
struct WebRtcInner {
    #[cfg(feature = "hydrate")]
    pc: Option<RtcPeerConnection>,
    has_audio: bool,
    friend_id: Option<i64>,
    client_id: Option<Uuid>,
    status: RwSignal<RtcStatus>,
}

impl WebRtcState {
    pub fn new() -> Self {
        let inner = WebRtcInner {
            #[cfg(feature = "hydrate")]
            pc: None,
            has_audio: false,
            friend_id: None,
            client_id: None,
            status: create_rw_signal(RtcStatus::Idle),
        };
        Self(store_value(inner))
    }

    /// Mute the audio track
    ///
    pub fn toggle_mute(&self, muted: RwSignal<bool>, toast: Toast) {
        let value = muted.get_untracked();
        if self.0.with_value(|v| v.has_audio) {
            #[cfg(feature = "hydrate")]
            if let Ok(el) = get_audio_element() {
                if let Some(media) = el.src_object() {
                    for track in media.get_tracks() {
                        let track = MediaStreamTrack::from(track);
                        track.set_enabled(value);
                    }
                    muted.set(!value);
                    return;
                }
            }
        }
        if !value {
            muted.set(true);
        }
        toast.error(String::from("No audio input found"))
    }

    /// Returns the rtc status
    ///
    pub fn status(&self) -> RwSignal<RtcStatus> {
        self.0.with_value(|v| v.status)
    }

    /// Returns the friend id of calling
    ///
    pub fn friend_id(&self) -> Option<i64> {
        self.0.with_value(|v| v.friend_id)
    }

    /// Returns the friend id of calling
    ///
    pub fn has_audio(&self) -> bool {
        self.0.with_value(|v| v.has_audio)
    }

    /// Send a call request to all peers of user B
    ///
    pub fn send_call(&self, friend_id: i64, ws: WebSocketState) {
        ws.send(Event::SendCall(friend_id));
    }

    /// Receive a notice that call has been send
    ///
    #[cfg(feature = "hydrate")]
    pub fn send_call_done(&self, friend_id: i64) {
        self.0.update_value(|v| {
            v.friend_id = Some(friend_id);
        });
        self.status().set(RtcStatus::Caller);
    }

    /// Receive a call request from peer A
    ///
    #[cfg(feature = "hydrate")]
    pub fn receive_call(&self, friend_id: i64, client_id: Uuid) {
        self.0.update_value(|v| {
            v.friend_id = Some(friend_id);
            v.client_id = Some(client_id);
        });
        self.status().set(RtcStatus::Callee);
    }

    /// Reject the call request and send hungup reson to all peers
    ///
    pub fn send_hung_up(&self, reson: HungUpReson, ws: WebSocketState) {
        if let Some(friend_id) = self.0.with_value(|v| v.friend_id.clone()) {
            ws.send(Event::SendHungUp(friend_id, reson));
        }
    }

    /// Receive hung up from A/B and reset all
    ///
    #[cfg(feature = "hydrate")]
    pub fn receive_hung_up(&self) {
        if let Some(pc) = self.0.with_value(|v| v.pc.clone()) {
            let _ = stop_tracks();
            pc.close();
        };
        self.0.update_value(|v| {
            v.friend_id = None;
            v.client_id = None;
            v.pc = None;
        });
        self.status().set(RtcStatus::Idle);
    }

    /// Accept the call request and send reply to peer A
    ///
    pub fn send_reply(&self, ws: WebSocketState) {
        let Some(friend_id) = self.0.with_value(|v| v.friend_id.clone()) else {
            return;
        };
        let Some(client_id) = self.0.with_value(|v: &WebRtcInner| v.client_id.clone()) else {
            return;
        };
        self.status().set(RtcStatus::Calling);

        ws.send(Event::SendReply(friend_id, client_id));
    }

    /// Receive reply from peer B and send offer back
    ///
    #[cfg(feature = "hydrate")]
    pub fn send_offer(&self, client_id: Uuid, ws: WebSocketState) {
        let rtc_ref = self.0;
        let Some(friend_id) = rtc_ref.with_value(|v| v.friend_id.clone()) else {
            return;
        };
        rtc_ref.update_value(|v| {
            v.client_id = Some(client_id);
        });
        self.status().set(RtcStatus::Calling);

        // create rtc peer connection for A
        let Ok(pc) = create_plain_connection() else {
            return;
        };
        rtc_ref.update_value(|v| {
            v.pc = Some(pc.clone());
        });

        spawn_local(async move {
            set_on_icecandidate(&pc, friend_id, client_id, ws);
            set_on_track(&pc).unwrap_throw();
            if add_local_stream(&pc).await.is_ok() {
                rtc_ref.update_value(|v| v.has_audio = true);
            }

            let offer = create_sdp_offer(&pc).await.unwrap_throw();
            ws.send(Event::SendOffer(friend_id, client_id, offer));
        });
    }

    /// Receive offer from peer A and send answer back
    ///
    #[cfg(feature = "hydrate")]
    pub fn send_answer(&self, offer: String, ws: WebSocketState) {
        let rtc_ref = self.0;
        let Some(friend_id) = rtc_ref.with_value(|v| v.friend_id.clone()) else {
            return;
        };
        let Some(client_id) = rtc_ref.with_value(|v: &WebRtcInner| v.client_id.clone()) else {
            return;
        };

        // create rtc peer connection for B
        let Ok(pc) = create_plain_connection() else {
            return;
        };
        rtc_ref.update_value(|v| {
            v.pc = Some(pc.clone());
        });

        spawn_local(async move {
            set_on_icecandidate(&pc, friend_id, client_id, ws);
            set_on_track(&pc).unwrap_throw();
            if add_local_stream(&pc).await.is_ok() {
                rtc_ref.update_value(|v| v.has_audio = true);
            }

            let answer = create_sdp_answer(&pc, offer).await.unwrap_throw();
            ws.send(Event::SendAnswer(friend_id, client_id, answer));
        });
    }

    /// Receive answer from peer B and store sdp info
    ///
    #[cfg(feature = "hydrate")]
    pub fn receive_answer(&self, answer: String) {
        if let Some(pc) = self.0.with_value(|v: &WebRtcInner| v.pc.clone()) {
            spawn_local(async move {
                let _ = store_sdp_answer(&pc, answer).await;
            })
        }
    }

    /// Receive ice candidate from A/B and add to peer connection
    ///
    #[cfg(feature = "hydrate")]
    pub fn receive_candidate(&self, candidate: IceCandidate) {
        if let Some(pc) = self.0.with_value(|v: &WebRtcInner| v.pc.clone()) {
            spawn_local(async move {
                let _ = add_ice_candidate(&pc, candidate).await;
            })
        }
    }
}

/// Create a rtc peer connection
///
#[cfg(feature = "hydrate")]
fn create_plain_connection() -> Result<RtcPeerConnection, JsValue> {
    RtcPeerConnection::new()
}

/// Set ice candidate callback to peer conn
///
#[cfg(feature = "hydrate")]
fn set_on_icecandidate(
    pc: &RtcPeerConnection,
    friend_id: i64,
    client_id: Uuid,
    ws: WebSocketState,
) {
    let onicecandidate_callback =
        Closure::<dyn FnMut(_)>::new(move |ev: RtcPeerConnectionIceEvent| {
            if let Some(rtc_candidate) = ev.candidate() {
                let ice_candidate = IceCandidate {
                    candidate: rtc_candidate.candidate(),
                    sdp_mid: rtc_candidate.sdp_mid().unwrap_or_default(),
                    sdp_m_line_index: rtc_candidate.sdp_m_line_index().unwrap_or_default(),
                };
                ws.send(Event::SendCandidate(friend_id, client_id, ice_candidate));
            }
        });
    pc.set_onicecandidate(Some(onicecandidate_callback.as_ref().unchecked_ref()));
    onicecandidate_callback.forget();
}

/// Get audio element from DOM by id "pcaudio"
///
#[cfg(feature = "hydrate")]
fn get_audio_element() -> Result<HtmlAudioElement, JsValue> {
    let audio_element = match document().get_element_by_id("pcaudio") {
        Some(el) => el,
        None => return Err(JsValue::from_str("No audio element found")),
    };

    let el = audio_element.dyn_into::<HtmlAudioElement>()?;
    Ok(el)
}

/// Set track callback to peer conn
///
#[cfg(feature = "hydrate")]
fn set_on_track(pc: &RtcPeerConnection) -> Result<(), JsValue> {
    let el = get_audio_element()?;

    let ontrack_callback = Closure::<dyn FnMut(_)>::new(move |ev: RtcTrackEvent| {
        let stream = ev.streams().get(0);
        let audio_stream = MediaStream::from(stream);
        el.set_src_object(Some(&audio_stream));
        let _ = el.play();
    });
    pc.set_ontrack(Some(ontrack_callback.as_ref().unchecked_ref()));
    ontrack_callback.forget();

    Ok(())
}

///
///
#[cfg(feature = "hydrate")]
fn stop_tracks() -> Result<(), JsValue> {
    let el = get_audio_element()?;
    if let Some(media) = el.src_object() {
        for track in media.get_tracks() {
            let track = MediaStreamTrack::from(track);
            track.stop();
        }
    }
    el.set_src_object(None);
    Ok(())
}

///
///
#[cfg(feature = "hydrate")]
async fn add_local_stream(pc: &RtcPeerConnection) -> Result<(), JsValue> {
    let media_devices = window().navigator().media_devices()?;

    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    constraints.set_video(&JsValue::FALSE);

    let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
    let ms = JsFuture::from(stream_promise).await?;
    let media_stream = MediaStream::from(ms);

    for track in media_stream.get_audio_tracks() {
        let track = MediaStreamTrack::from(track);
        pc.add_track_0(&track, &media_stream);
    }

    Ok(())
}

/// Create sdp offer
///
#[cfg(feature = "hydrate")]
async fn create_sdp_offer(pc: &RtcPeerConnection) -> Result<String, JsValue> {
    let options = RtcOfferOptions::new();
    options.set_offer_to_receive_audio(true);

    let offer_promise = pc.create_offer_with_rtc_offer_options(&options);
    let offer_js = JsFuture::from(offer_promise).await?;
    let offer = Reflect::get(&offer_js, &JsValue::from_str("sdp"))?
        .as_string()
        .unwrap();

    let offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.set_sdp(&offer);

    let sld_promise = pc.set_local_description(&offer_obj);
    JsFuture::from(sld_promise).await?;

    Ok(offer)
}

/// Create sdp answer
///
#[cfg(feature = "hydrate")]
async fn create_sdp_answer(pc: &RtcPeerConnection, offer: String) -> Result<String, JsValue> {
    let offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.set_sdp(&offer);
    let srd_promise = pc.set_remote_description(&offer_obj);
    JsFuture::from(srd_promise).await?;

    // create SDP Answer
    let answer_js = JsFuture::from(pc.create_answer()).await?;
    let answer = Reflect::get(&answer_js, &JsValue::from_str("sdp"))?
        .as_string()
        .unwrap();

    let answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    answer_obj.set_sdp(&answer);

    let sld_promise = pc.set_local_description(&answer_obj);
    JsFuture::from(sld_promise).await?;

    Ok(answer)
}

/// Store remote answer desc
///
#[cfg(feature = "hydrate")]
async fn store_sdp_answer(pc: &RtcPeerConnection, answer: String) -> Result<(), JsValue> {
    // setting remote description
    let answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    answer_obj.set_sdp(&answer);
    let srd_promise = pc.set_remote_description(&answer_obj);
    JsFuture::from(srd_promise).await?;
    Ok(())
}

/// Add ice candidate with opt
///
#[cfg(feature = "hydrate")]
async fn add_ice_candidate(pc: &RtcPeerConnection, candidate: IceCandidate) -> Result<(), JsValue> {
    // setting remote description
    let candidate_init_dict = RtcIceCandidateInit::new("");
    candidate_init_dict.set_candidate(&candidate.candidate);
    candidate_init_dict.set_sdp_m_line_index(Some(candidate.sdp_m_line_index));
    candidate_init_dict.set_sdp_mid(Some(&candidate.sdp_mid));

    if let Ok(candidate) = RtcIceCandidate::new(&candidate_init_dict) {
        let cand_promise = pc.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate));
        JsFuture::from(cand_promise).await?;
    }
    Ok(())
}
