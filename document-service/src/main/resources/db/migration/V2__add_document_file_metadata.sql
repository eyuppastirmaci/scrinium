ALTER TABLE documents
    ADD COLUMN content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    ADD COLUMN size_bytes BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN storage_object_key TEXT NOT NULL DEFAULT 'legacy/unavailable',
    ADD COLUMN sha256 TEXT;

ALTER TABLE documents
    ADD CONSTRAINT documents_content_type_not_blank CHECK (btrim(content_type) <> ''),
    ADD CONSTRAINT documents_size_bytes_non_negative CHECK (size_bytes >= 0),
    ADD CONSTRAINT documents_storage_object_key_not_blank CHECK (btrim(storage_object_key) <> ''),
    ADD CONSTRAINT documents_sha256_not_blank_when_present CHECK (sha256 IS NULL OR btrim(sha256) <> '');
