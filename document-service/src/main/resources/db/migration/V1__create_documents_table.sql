CREATE TABLE documents (
                           id         UUID        PRIMARY KEY,
                           file_name  TEXT        NOT NULL,
                           status     TEXT        NOT NULL CHECK (status IN ('PENDING', 'READY')),
                           created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                           updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);