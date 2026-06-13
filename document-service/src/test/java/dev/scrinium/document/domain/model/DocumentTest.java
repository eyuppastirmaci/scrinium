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

        Document document = Document.pending(id, "invoice.pdf", now);

        assertThat(document.id()).isEqualTo(id);
        assertThat(document.fileName()).isEqualTo("invoice.pdf");
        assertThat(document.status()).isEqualTo(DocumentStatus.PENDING);
        // pending() must use the same instant for both timestamps.
        assertThat(document.createdAt()).isEqualTo(now);
        assertThat(document.updatedAt()).isEqualTo(now);
    }

    @Test
    void construction_rejectsBlankFileName() {
        assertThatThrownBy(() ->
                Document.pending(UUID.randomUUID(), "   ", OffsetDateTime.now()))
                .isInstanceOf(InvalidDocumentException.class)
                .hasMessageContaining("fileName");
    }

    @Test
    void construction_rejectsNullFileName() {
        assertThatThrownBy(() ->
                Document.pending(UUID.randomUUID(), null, OffsetDateTime.now()))
                .isInstanceOf(InvalidDocumentException.class);
    }

    @Test
    void construction_rejectsNullId() {
        // Structural invariant (programming error) -> NPE, not a domain exception.
        assertThatThrownBy(() ->
                new Document(null, "invoice.pdf", DocumentStatus.PENDING,
                        OffsetDateTime.now(), OffsetDateTime.now()))
                .isInstanceOf(NullPointerException.class);
    }

    @Test
    void construction_acceptsValidArguments() {
        assertThatNoException().isThrownBy(() ->
                Document.pending(UUID.randomUUID(), "invoice.pdf", OffsetDateTime.now()));
    }
}