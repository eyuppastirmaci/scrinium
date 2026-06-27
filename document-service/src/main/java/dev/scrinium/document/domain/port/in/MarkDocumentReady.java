package dev.scrinium.document.domain.port.in;

import dev.scrinium.document.domain.model.DocumentMetadata;
import dev.scrinium.document.domain.model.DocumentThumbnail;
import dev.scrinium.document.domain.model.ExtractedPage;

import java.util.List;
import java.util.UUID;

public interface MarkDocumentReady {

    record ProcessingResult(
            UUID documentId,
            List<ExtractedPage> pages,
            DocumentMetadata metadata,
            List<DocumentThumbnail> thumbnails
    ) {}

    void markReady(ProcessingResult result);
}