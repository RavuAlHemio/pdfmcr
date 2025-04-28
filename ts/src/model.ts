export interface PageAnnotations {
    annotations: Annotation[];
    artifacts: Artifact[];
}

export type ArtifactKind = "Pagination"|"Page"|"Layout"|"Background";

export interface Artifact {
    kind: ArtifactKind;
    annotation: Annotation;
}

export interface Annotation {
    left: number;
    bottom: number;
    elements: TextChunk[];
}

export type FontVariant = "Regular"|"Italic"|"Bold"|"BoldItalic";

export interface TextChunk {
    text: string;
    font_variant: FontVariant;
    font_size: number;
    character_spacing: number;
    word_spacing: number;
    leading: number;
    language: string|null;
    alternate_text: string|null;
    actual_text: string|null;
    expansion: string|null;
}
