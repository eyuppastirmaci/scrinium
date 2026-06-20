CREATE TABLE processing_jobs (
    document_id UUID PRIMARY KEY,
    status TEXT NOT NULL,
    file_name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_object_key TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT processing_jobs_status_valid
        CHECK (status IN ('RECEIVED', 'PROCESSING', 'COMPLETED', 'FAILED')),
    CONSTRAINT processing_jobs_file_name_not_blank
        CHECK (btrim(file_name) <> ''),
    CONSTRAINT processing_jobs_content_type_not_blank
        CHECK (btrim(content_type) <> ''),
    CONSTRAINT processing_jobs_size_bytes_non_negative
        CHECK (size_bytes >= 0),
    CONSTRAINT processing_jobs_storage_object_key_not_blank
        CHECK (btrim(storage_object_key) <> ''),
    CONSTRAINT processing_jobs_sha256_not_blank
        CHECK (btrim(sha256) <> ''),
    CONSTRAINT processing_jobs_attempts_non_negative
        CHECK (attempts >= 0),
    CONSTRAINT processing_jobs_last_error_not_blank_when_present
        CHECK (last_error IS NULL OR btrim(last_error) <> '')
);

CREATE INDEX processing_jobs_status_idx ON processing_jobs (status);
CREATE INDEX processing_jobs_sha256_idx ON processing_jobs (sha256);
