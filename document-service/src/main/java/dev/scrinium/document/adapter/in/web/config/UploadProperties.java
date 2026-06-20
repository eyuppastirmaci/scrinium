package dev.scrinium.document.adapter.in.web.config;

import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Positive;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.util.unit.DataSize;
import org.springframework.validation.annotation.Validated;

import java.util.Set;

@Validated
@ConfigurationProperties(prefix = "scrinium.upload")
public record UploadProperties(
        @NotNull DataSize maxFileSize,
        @NotEmpty Set<String> supportedContentTypes,
        @Positive int maxFilesPerRequest
) {}
