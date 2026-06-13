package dev.scrinium.document.domain.port.out;

import dev.scrinium.document.domain.model.Document;

public interface DocumentEventPublisher {
    void documentUploaded(Document document);
}