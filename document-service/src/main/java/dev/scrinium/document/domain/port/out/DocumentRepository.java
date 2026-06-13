package dev.scrinium.document.domain.port.out;

import dev.scrinium.document.domain.model.Document;

import java.util.UUID;

public interface DocumentRepository {
    void save(Document document);

    /**
     * Transitions the document to READY only if it is currently PENDING.
     * @return number of affected rows (1 = transitioned, 0 = already READY or unknown id)
     */
    int markReadyIfPending(UUID documentId);
}
