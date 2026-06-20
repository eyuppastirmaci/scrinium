package dev.scrinium.document.adapter.out.persistence;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

import java.util.UUID;

@Repository
public class JdbcDocumentRepository implements DocumentRepository {

    private final JdbcClient jdbcClient;

    public JdbcDocumentRepository(JdbcClient jdbcClient) {
        this.jdbcClient = jdbcClient;
    }

    @Override
    public void save(Document document) {
        jdbcClient.sql("""
                INSERT INTO documents (
                    id,
                    file_name,
                    content_type,
                    size_bytes,
                    storage_object_key,
                    sha256,
                    status,
                    created_at,
                    updated_at
                )
                VALUES (
                    :id,
                    :fileName,
                    :contentType,
                    :sizeBytes,
                    :storageObjectKey,
                    :sha256,
                    :status,
                    :createdAt,
                    :updatedAt
                )
                """)
                .param("id", document.id())
                .param("fileName", document.fileName())
                .param("contentType", document.contentType())
                .param("sizeBytes", document.sizeBytes())
                .param("storageObjectKey", document.storageObjectKey())
                .param("sha256", document.sha256())
                .param("status", document.status().name())
                .param("createdAt", document.createdAt())
                .param("updatedAt", document.updatedAt())
                .update();
    }

    @Override
    public int markReadyIfPending(UUID documentId) {
        return jdbcClient.sql("""
            UPDATE documents
               SET status = 'READY', updated_at = now()
             WHERE id = :id AND status = 'PENDING'
            """)
                .param("id", documentId)
                .update();
    }
}
