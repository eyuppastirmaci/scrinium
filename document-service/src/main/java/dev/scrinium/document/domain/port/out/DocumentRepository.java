package dev.scrinium.document.domain.port.out;

import dev.scrinium.document.domain.model.Document;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface DocumentRepository {
    void save(Document document);

    Optional<Document> findById(UUID documentId);

    Optional<Document> findBySha256(String sha256);

    /**
     * Transitions the document to READY only if it is currently PENDING.
     * @return number of affected rows (1 = transitioned, 0 = already READY or unknown id)
     */
    int markReadyIfPending(UUID documentId);

    int markFailedIfPending(UUID documentId, String reason);

    int markDeleted(UUID documentId);

    List<Document> findAll(int offset, int limit);

    long countAccessible();
}
