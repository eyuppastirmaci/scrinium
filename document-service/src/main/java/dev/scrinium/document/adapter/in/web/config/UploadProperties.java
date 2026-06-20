package dev.scrinium.document.adapter.in.web.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

import java.util.Set;

@ConfigurationProperties(prefix = "scrinium.upload")
public record UploadProperties(Set<String> supportedContentTypes) {

    public UploadProperties {
        if (supportedContentTypes == null || supportedContentTypes.isEmpty()) {
            supportedContentTypes = Set.of(
                    "application/pdf",
                    "image/jpeg",
                    "image/png",
                    "image/webp",
                    "image/tiff"
            );
        } else {
            supportedContentTypes = Set.copyOf(supportedContentTypes);
        }
    }
}
