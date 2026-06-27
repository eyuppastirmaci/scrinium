package dev.scrinium.document.domain.model;

import dev.scrinium.document.domain.exception.InvalidDocumentException;
import org.junit.jupiter.api.Test;

import java.time.OffsetDateTime;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.assertj.core.api.Assertions.assertThatNoException;

class DocumentTest {

    @Test
    void pending_createsDocumentInPendingStatus() {
        UUID id = UUID.randomUUID();
        OffsetDateTime now = OffsetDateTime.now();

        Document document = Document.pending(
                id,
                "invoice.pdf",
                "application/pdf",
                1_024,
                "documents/%s/invoice.pdf".formatted(id),
                "abc123",
                now
        );

        assertThat(document.id()).isEqualTo(id);
        assertThat(document.fileName()).isEqualTo("invoice.pdf");
        assertThat(document.contentType()).isEqualTo("application/pdf");
        assertThat(document.sizeBytes()).isEqualTo(1_024);
        assertThat(document.storageObjectKey()).isEqualTo("documents/%s/invoice.pdf".formatted(id));
        assertThat(document.sha256()).isEqualTo("abc123");
        assertThat(document.status()).isEqualTo(DocumentStatus.PENDING);
        // pending() must use the same instant for both timestamps.
        assertThat(document.createdAt()).isEqualTo(now);
        assertThat(document.updatedAt()).isEqualTo(now);
    }

    @Test
    void construction_rejectsBlankFileName() {
        assertThatThrownBy(() ->
                pendingDocument("   ", "application/pdf", 1_024, "documents/id/invoice.pdf", null))
                .isInstanceOf(InvalidDocumentException.class)
                .hasMessageContaining("fileName");
    }

    @Test
    void construction_rejectsNullFileName() {
        assertThatThrownBy(() ->
                pendingDocument(null, "application/pdf", 1_024, "documents/id/invoice.pdf", null))
                .isInstanceOf(InvalidDocumentException.class);
    }

    @Test
    void construction_rejectsNullId() {
        // Structural invariant (programming error) -> NPE, not a domain exception.
        assertThatThrownBy(() ->
                new Document(null, "invoice.pdf", "application/pdf", 1_024,
                        "documents/id/invoice.pdf", null, DocumentStatus.PENDING, null,
                        OffsetDateTime.now(), OffsetDateTime.now()))
                .isInstanceOf(NullPointerException.class);
    }

    @Test
    void construction_rejectsBlankContentType() {
        assertThatThrownBy(() ->
                pendingDocument("invoice.pdf", "   ", 1_024, "documents/id/invoice.pdf", null))
                .isInstanceOf(InvalidDocumentException.class)
                .hasMessageContaining("contentType");
    }

    @Test
    void construction_rejectsNonPositiveSize() {
        assertThatThrownBy(() ->
                pendingDocument("invoice.pdf", "application/pdf", 0, "documents/id/invoice.pdf", null))
                .isInstanceOf(InvalidDocumentException.class)
                .hasMessageContaining("sizeBytes");
    }

    @Test
    void construction_rejectsBlankStorageObjectKey() {
        assertThatThrownBy(() ->
                pendingDocument("invoice.pdf", "application/pdf", 1_024, "   ", null))
                .isInstanceOf(InvalidDocumentException.class)
                .hasMessageContaining("storageObjectKey");
    }

    @Test
    void construction_rejectsBlankSha256WhenPresent() {
        assertThatThrownBy(() ->
                pendingDocument("invoice.pdf", "application/pdf", 1_024, "documents/id/invoice.pdf", "   "))
                .isInstanceOf(InvalidDocumentException.class)
                .hasMessageContaining("sha256");
    }

    @Test
    void construction_acceptsValidArguments() {
        assertThatNoException().isThrownBy(() ->
                pendingDocument("invoice.pdf", "application/pdf", 1_024, "documents/id/invoice.pdf", null));
    }

    private Document pendingDocument(
            String fileName,
            String contentType,
            long sizeBytes,
            String storageObjectKey,
            String sha256
    ) {
        return Document.pending(
                UUID.randomUUID(),
                fileName,
                contentType,
                sizeBytes,
                storageObjectKey,
                sha256,
                OffsetDateTime.now()
        );
    }
}
