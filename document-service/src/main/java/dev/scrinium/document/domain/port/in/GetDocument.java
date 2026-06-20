package dev.scrinium.document.domain.port.in;

import dev.scrinium.document.domain.model.Document;

import java.util.UUID;

public interface GetDocument {
    Document get(UUID documentId);
}
