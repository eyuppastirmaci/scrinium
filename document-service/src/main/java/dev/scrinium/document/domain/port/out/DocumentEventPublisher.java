package dev.scrinium.document.domain.port.out;

import dev.scrinium.document.domain.model.Document;

import java.util.UUID;

public interface DocumentEventPublisher {
    void documentUploaded(Document document);

    void documentDeleted(UUID documentId);
}