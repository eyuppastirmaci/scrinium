package dev.scrinium.document.domain.port.in;

import java.util.UUID;

public interface MarkDocumentFailed {
    void markFailed(UUID documentId);
}
