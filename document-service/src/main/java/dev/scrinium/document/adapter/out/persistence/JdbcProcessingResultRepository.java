package dev.scrinium.document.adapter.out.persistence;

import dev.scrinium.document.domain.model.DocumentMetadata;
import dev.scrinium.document.domain.model.DocumentThumbnail;
import dev.scrinium.document.domain.model.ExtractedPage;
import dev.scrinium.document.domain.port.out.ProcessingResultRepository;
import org.springframework.jdbc.core.simple.JdbcClient;
import org.springframework.stereotype.Repository;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

@Repository
public class JdbcProcessingResultRepository implements ProcessingResultRepository {

    private final JdbcClient jdbcClient;

    public JdbcProcessingResultRepository(JdbcClient jdbcClient) {
        this.jdbcClient = jdbcClient;
    }

    @Override
    public void saveExtractedPages(UUID documentId, List<ExtractedPage> pages) {
        for (ExtractedPage page : pages) {
            jdbcClient.sql("""
                INSERT INTO extracted_pages (document_id, page_number, extracted_text)
                VALUES (:documentId, :pageNumber, :text)
                ON CONFLICT (document_id, page_number) DO UPDATE
                SET extracted_text = EXCLUDED.extracted_text
                """)
                    .param("documentId", documentId)
                    .param("pageNumber", page.pageNumber())
                    .param("text", page.text())
                    .update();
        }
    }

    @Override
    public List<ExtractedPage> findExtractedPages(UUID documentId) {
        return jdbcClient.sql("""
                SELECT page_number, extracted_text
                  FROM extracted_pages
                 WHERE document_id = :documentId
                 ORDER BY page_number
                """)
                .param("documentId", documentId)
                .query((rs, _) -> new ExtractedPage(
                        rs.getInt("page_number"),
                        rs.getString("extracted_text")
                ))
                .list();
    }

    @Override
    public void saveMetadata(DocumentMetadata metadata) {
        jdbcClient.sql("""
                INSERT INTO document_metadata (
                    document_id, page_count,
                    pdf_created_at, pdf_modified_at, pdf_author, pdf_title,
                    image_captured_at, image_device, image_gps_present, image_gps_redacted,
                    detected_language
                )
                VALUES (
                    :documentId, :pageCount,
                    :pdfCreatedAt, :pdfModifiedAt, :pdfAuthor, :pdfTitle,
                    :imageCapturedAt, :imageDevice, :imageGpsPresent, :imageGpsRedacted,
                    :detectedLanguage
                )
                ON CONFLICT (document_id) DO UPDATE
                SET page_count = EXCLUDED.page_count,
                    pdf_created_at = EXCLUDED.pdf_created_at,
                    pdf_modified_at = EXCLUDED.pdf_modified_at,
                    pdf_author = EXCLUDED.pdf_author,
                    pdf_title = EXCLUDED.pdf_title,
                    image_captured_at = EXCLUDED.image_captured_at,
                    image_device = EXCLUDED.image_device,
                    image_gps_present = EXCLUDED.image_gps_present,
                    image_gps_redacted = EXCLUDED.image_gps_redacted,
                    detected_language = EXCLUDED.detected_language,
                    updated_at = now()
                """)
                .param("documentId", metadata.documentId())
                .param("pageCount", metadata.pageCount())
                .param("pdfCreatedAt", metadata.pdfCreatedAt())
                .param("pdfModifiedAt", metadata.pdfModifiedAt())
                .param("pdfAuthor", metadata.pdfAuthor())
                .param("pdfTitle", metadata.pdfTitle())
                .param("imageCapturedAt", metadata.imageCapturedAt())
                .param("imageDevice", metadata.imageDevice())
                .param("imageGpsPresent", metadata.imageGpsPresent())
                .param("imageGpsRedacted", metadata.imageGpsRedacted())
                .param("detectedLanguage", metadata.detectedLanguage())
                .update();
    }

    @Override
    public Optional<DocumentMetadata> findMetadata(UUID documentId) {
        return jdbcClient.sql("""
                SELECT document_id, page_count,
                       pdf_created_at, pdf_modified_at, pdf_author, pdf_title,
                       image_captured_at, image_device, image_gps_present, image_gps_redacted,
                       detected_language
                  FROM document_metadata
                 WHERE document_id = :documentId
                """)
                .param("documentId", documentId)
                .query((rs, _) -> new DocumentMetadata(
                        rs.getObject("document_id", UUID.class),
                        rs.getObject("page_count", Integer.class),
                        rs.getObject("pdf_created_at", OffsetDateTime.class),
                        rs.getObject("pdf_modified_at", OffsetDateTime.class),
                        rs.getString("pdf_author"),
                        rs.getString("pdf_title"),
                        rs.getObject("image_captured_at", OffsetDateTime.class),
                        rs.getString("image_device"),
                        rs.getBoolean("image_gps_present"),
                        rs.getBoolean("image_gps_redacted"),
                        rs.getString("detected_language")
                ))
                .optional();
    }

    @Override
    public void saveThumbnails(UUID documentId, List<DocumentThumbnail> thumbnails) {
        for (DocumentThumbnail thumb : thumbnails) {
            jdbcClient.sql("""
                INSERT INTO document_thumbnails (document_id, size, storage_key, width, height)
                VALUES (:documentId, :size, :storageKey, :width, :height)
                ON CONFLICT (document_id, size) DO UPDATE
                SET storage_key = EXCLUDED.storage_key,
                    width = EXCLUDED.width,
                    height = EXCLUDED.height
                """)
                    .param("documentId", documentId)
                    .param("size", thumb.size())
                    .param("storageKey", thumb.storageKey())
                    .param("width", thumb.width())
                    .param("height", thumb.height())
                    .update();
        }
    }

    @Override
    public Optional<DocumentThumbnail> findThumbnail(UUID documentId, String size) {
        return jdbcClient.sql("""
                SELECT document_id, size, storage_key, width, height
                  FROM document_thumbnails
                 WHERE document_id = :documentId AND size = :size
                """)
                .param("documentId", documentId)
                .param("size", size)
                .query((rs, _) -> new DocumentThumbnail(
                        rs.getObject("document_id", UUID.class),
                        rs.getString("size"),
                        rs.getString("storage_key"),
                        rs.getInt("width"),
                        rs.getInt("height")
                ))
                .optional();
    }

    @Override
    public List<DocumentThumbnail> findThumbnails(UUID documentId) {
        return jdbcClient.sql("""
                SELECT document_id, size, storage_key, width, height
                  FROM document_thumbnails
                 WHERE document_id = :documentId
                """)
                .param("documentId", documentId)
                .query((rs, _) -> new DocumentThumbnail(
                        rs.getObject("document_id", UUID.class),
                        rs.getString("size"),
                        rs.getString("storage_key"),
                        rs.getInt("width"),
                        rs.getInt("height")
                ))
                .list();
    }

    @Override
    public void deleteAll(UUID documentId) {
        jdbcClient.sql("DELETE FROM extracted_pages WHERE document_id = :id")
                .param("id", documentId).update();
        jdbcClient.sql("DELETE FROM document_thumbnails WHERE document_id = :id")
                .param("id", documentId).update();
        jdbcClient.sql("DELETE FROM document_metadata WHERE document_id = :id")
                .param("id", documentId).update();
    }
}
