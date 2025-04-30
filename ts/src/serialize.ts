import { getImageHeightPt, pointsValue, positionFromTranslate, SVG_NS } from "./common";
import { Annotation, ArtifactKind, PageAnnotations, TextChunk } from "./model";


// keep this in sync with src/model.rs, obviously
export namespace Serialize {
    function serializeAnnotation(annotationGroup: SVGGElement, imageHeightPt: number): Annotation|null {
        const svgRoot = annotationGroup.ownerSVGElement;
        if (svgRoot === null) {
            return null;
        }

        const pos = positionFromTranslate(annotationGroup, SVGLength.SVG_LENGTHTYPE_PT);
        if (pos === null) {
            return null;
        }

        const textChildren = annotationGroup.getElementsByTagNameNS(SVG_NS, "text");
        if (textChildren.length === 0) {
            return null;
        }
        const textChild = textChildren[0];

        const elements: TextChunk[] = [];
        for (let rawChild of textChild.children) {
            if (rawChild.namespaceURI !== SVG_NS) {
                continue;
            }
            if (rawChild.localName !== "tspan") {
                continue;
            }
            const tspan = <SVGTSpanElement>rawChild;

            const fontSizePt = pointsValue(tspan.style.fontSize);
            const characterSpacingPt = pointsValue(tspan.style.letterSpacing);
            const wordSpacingPt = pointsValue(tspan.style.wordSpacing);
            const lineHeightPt = pointsValue(tspan.style.lineHeight);
            const isBold = tspan.style.fontWeight === "bold";
            const isItalic = tspan.style.fontStyle === "italic";

            if (fontSizePt === null) {
                continue;
            }
            if (characterSpacingPt === null) {
                continue;
            }
            if (wordSpacingPt === null) {
                continue;
            }
            if (lineHeightPt === null) {
                continue;
            }

            const leadingPt = lineHeightPt - fontSizePt;

            const children = tspan.childNodes;
            if (children.length !== 1) {
                return null;
            }
            if (children[0].nodeType !== Node.TEXT_NODE) {
                return null;
            }
            const text = (<Text>children[0]).textContent;
            if (text === null) {
                return null;
            }

            const fontVariant = isBold
                ? (isItalic ? "BoldItalic" : "Bold")
                : (isItalic ? "Italic" : "Regular");

            const language = tspan.getAttribute("data-lang");
            const alternate_text = tspan.getAttribute("data-alt-text");
            const actual_text = tspan.getAttribute("data-actual-text");
            const expansion = tspan.getAttribute("data-expansion");

            elements.push({
                text,
                font_variant: fontVariant,
                font_size: Math.round(fontSizePt),
                character_spacing: Math.round(characterSpacingPt),
                word_spacing: Math.round(wordSpacingPt),
                leading: Math.round(leadingPt),
                language,
                alternate_text,
                actual_text,
                expansion,
            });
        }

        return {
            left: Math.round(pos.x),
            bottom: Math.round(imageHeightPt - pos.y),
            elements,
        };
    }

    export function serialize(pageGroup: SVGGElement): PageAnnotations {
        const ret: PageAnnotations = {
            annotations: [],
            artifacts: [],
        };

        const imageHeightPt = getImageHeightPt(pageGroup);

        for (let child of pageGroup.children) {
            if (child.namespaceURI !== SVG_NS) {
                continue;
            }

            if (child.localName === "g") {
                const gChild = <SVGGElement>child;
                if (gChild.classList.contains("annotation")) {
                    if (imageHeightPt === null) {
                        continue;
                    }

                    // it's an annotation!
                    const annotation = serializeAnnotation(gChild, imageHeightPt);
                    if (annotation === null) {
                        continue;
                    }
                    ret.annotations.push(annotation);
                } else if (gChild.classList.contains("artifact")) {
                    if (imageHeightPt === null) {
                        continue;
                    }

                    let artifactKind: ArtifactKind|null = null;
                    if (gChild.classList.contains("background")) {
                        artifactKind = "Background";
                    } else if (gChild.classList.contains("layout")) {
                        artifactKind = "Layout";
                    } else if (gChild.classList.contains("page")) {
                        artifactKind = "Page";
                    } else if (gChild.classList.contains("pagination")) {
                        artifactKind = "Pagination";
                    } else {
                        continue;
                    }
                    const annotation = serializeAnnotation(gChild, imageHeightPt);
                    if (annotation === null) {
                        continue;
                    }
                    ret.artifacts.push({
                        kind: artifactKind,
                        annotation,
                    });
                }
            }
        }

        return ret;
    }

    async function doSave(): Promise<void> {
        // find the page group
        const pageGroup = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (pageGroup === null) {
            alert("cannot save: page group not found");
            return;
        }

        // find the number of the page
        const metaElement = <HTMLMetaElement|null>document.querySelector("meta[name=\"pdfmcr-page-number\"]");
        if (metaElement === null) {
            alert("cannot save: page number meta element not found");
            return;
        }
        const pageNumber = +metaElement.content;

        const pageAnnotations = serialize(pageGroup);
        const request = new Request(
            `/page/${pageNumber}/annotations`,
            {
                method: "POST",
                body: JSON.stringify(pageAnnotations),
                headers: {
                    "Content-Type": "application/json",
                },
            },
        );
        let success = false;
        try {
            const response = await fetch(request);
            const responseText = response.text();
            if (response.status !== 200) {
                alert("cannot save: " + responseText);
            } else {
                success = true;
            }
        } catch (errorPromise) {
            let error = await errorPromise;
            alert("cannot save: " + error);
        }
        if (success) {
            alert("saved!");
        }
    }

    function doInit(): void {
        const saveButton = <HTMLInputElement|null>document.getElementById("pdfmcr-save-button");
        if (saveButton === null) {
            return;
        }

        saveButton.addEventListener("click", doSave);
    }

    export function init(): void {
        document.addEventListener("DOMContentLoaded", doInit);
    }
}
