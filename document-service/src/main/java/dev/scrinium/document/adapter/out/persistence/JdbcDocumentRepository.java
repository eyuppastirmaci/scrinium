package dev.scrinium.document.adapter.out.persistence;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentStatus;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.springframework.jdbc.core.RowMapper;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Optional;
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
    public Optional<Document> findById(UUID documentId) {
        return jdbcClient.sql("""
                SELECT id, file_name, content_type, size_bytes,
                       storage_object_key, sha256, status,
                       created_at, updated_at
                  FROM documents
                 WHERE id = :id AND status IN ('PENDING', 'READY')
                """)
                .param("id", documentId)
                .query(DOCUMENT_ROW_MAPPER)
                .optional();
    }

    @Override
    public Optional<Document> findBySha256(String sha256) {
        return jdbcClient.sql("""
                SELECT id, file_name, content_type, size_bytes,
                       storage_object_key, sha256, status,
                       created_at, updated_at
                  FROM documents
                 WHERE sha256 = :sha256 AND status IN ('PENDING', 'READY')
                 LIMIT 1
                """)
                .param("sha256", sha256)
                .query(DOCUMENT_ROW_MAPPER)
                .optional();
    }

    @Override
    public int markDeleted(UUID documentId) {
        return jdbcClient.sql("""
            UPDATE documents
               SET status = 'DELETED', updated_at = now()
             WHERE id = :id AND status IN ('PENDING', 'READY', 'FAILED')
            """)
                .param("id", documentId)
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

    @Override
    public int markFailedIfPending(UUID documentId) {
        return jdbcClient.sql("""
            UPDATE documents
               SET status = 'FAILED', updated_at = now()
             WHERE id = :id AND status = 'PENDING'
            """)
                .param("id", documentId)
                .update();
    }

    @Override
    public List<Document> findAll(int offset, int limit) {
        return jdbcClient.sql("""
                SELECT id, file_name, content_type, size_bytes,
                       storage_object_key, sha256, status,
                       created_at, updated_at
                  FROM documents
                 WHERE status IN ('PENDING', 'READY', 'FAILED')
                 ORDER BY created_at DESC
                 LIMIT :limit OFFSET :offset
                """)
                .param("offset", offset)
                .param("limit", limit)
                .query(DOCUMENT_ROW_MAPPER)
                .list();
    }

    @Override
    public long countAccessible() {
        return jdbcClient.sql("""
                SELECT COUNT(*) FROM documents
                 WHERE status IN ('PENDING', 'READY', 'FAILED')
                """)
                .query(Long.class)
                .single();
    }

    private static final RowMapper<Document> DOCUMENT_ROW_MAPPER = (rs, _) ->
            new Document(
                    rs.getObject("id", UUID.class),
                    rs.getString("file_name"),
                    rs.getString("content_type"),
                    rs.getLong("size_bytes"),
                    rs.getString("storage_object_key"),
                    rs.getString("sha256"),
                    DocumentStatus.valueOf(rs.getString("status")),
                    rs.getObject("created_at", OffsetDateTime.class),
                    rs.getObject("updated_at", OffsetDateTime.class)
            );
}
