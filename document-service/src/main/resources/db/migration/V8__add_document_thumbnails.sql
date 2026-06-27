CREATE TABLE document_thumbnails (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    size TEXT NOT NULL,
    storage_key TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT document_thumbnails_unique_size UNIQUE (document_id, size),
    CONSTRAINT document_thumbnails_size_valid CHECK (size IN ('SMALL', 'MEDIUM')),
    CONSTRAINT document_thumbnails_storage_key_not_blank CHECK (btrim(storage_key) <> ''),
    CONSTRAINT document_thumbnails_width_positive CHECK (width > 0),
    CONSTRAINT document_thumbnails_height_positive CHECK (height > 0)
);
