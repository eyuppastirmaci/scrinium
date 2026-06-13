package dev.scrinium.document.domain.port.in;

import java.util.UUID;

public interface MarkDocumentReady {
    void markReady(UUID documentId);
}