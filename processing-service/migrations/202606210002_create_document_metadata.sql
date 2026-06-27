CREATE TABLE document_metadata (
    document_id UUID PRIMARY KEY REFERENCES processing_jobs(document_id) ON DELETE CASCADE,
    page_count INTEGER,
    pdf_created_at TIMESTAMPTZ,
    pdf_modified_at TIMESTAMPTZ,
    pdf_author TEXT,
    pdf_title TEXT,
    image_captured_at TIMESTAMPTZ,
    image_device TEXT,
    image_gps_present BOOLEAN NOT NULL DEFAULT false,
    image_gps_redacted BOOLEAN NOT NULL DEFAULT false,
    detected_language TEXT,
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT document_metadata_page_count_positive
        CHECK (page_count IS NULL OR page_count > 0),
    CONSTRAINT document_metadata_detected_language_not_blank
        CHECK (detected_language IS NULL OR length(trim(detected_language)) > 0)
);

CREATE INDEX document_metadata_pdf_created_at_idx
    ON document_metadata (pdf_created_at)
    WHERE pdf_created_at IS NOT NULL;

CREATE INDEX document_metadata_image_captured_at_idx
    ON document_metadata (image_captured_at)
    WHERE image_captured_at IS NOT NULL;

CREATE INDEX document_metadata_detected_language_idx
    ON document_metadata (detected_language)
    WHERE detected_language IS NOT NULL;
