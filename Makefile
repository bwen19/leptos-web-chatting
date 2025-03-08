DATABASE_URL="sqlite://db/chat.db"
MIGRATIONS_DIR="./common/migrations"

dev:
	LEPTOS_TAILWIND_VERSION="v3.4.10" cargo leptos watch

createdb:
	sqlx database create --database-url ${DATABASE_URL}

dropdb:
	sqlx database drop --database-url ${DATABASE_URL}

migration:
	sqlx migrate add --sequential --source ${MIGRATIONS_DIR} -r $(name)

migrateup:
	sqlx migrate run --database-url ${DATABASE_URL} --source ${MIGRATIONS_DIR}

migratedown:
	sqlx migrate revert --database-url ${DATABASE_URL} --source ${MIGRATIONS_DIR}

redis:
	docker run --name redis -p 6379:6379 -d redis:7.2.5-alpine3.20 redis-server --requirepass "secret"

container:
	docker build . -t bwen19/chat

.PHONY: dev createdb dropdb migration migrateup migratedown redis container
