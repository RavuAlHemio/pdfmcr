import { getImageHeightPt, Position, positionFromTranslate, SVG_NS } from "./common";
import { Annotation, ArtifactKind, PageAnnotations, TextChunk } from "./model";

export namespace Annotations {
    let dragStart: Position|null = null;
    let dragEvents: ["mousemove"|"mouseup", any][] = [];

    function setGroupPos(annotationGroup: SVGGElement, clientX: number, clientY: number): SVGSVGElement|null {
        if (dragStart === null) {
            return null;
        }

        const svgRoot = annotationGroup.ownerSVGElement;
        if (svgRoot === null) {
            return null;
        }

        const transform = svgRoot.createSVGTransform();
        transform.setTranslate(
            dragStart.x + clientX,
            dragStart.y + clientY,
        );
        annotationGroup.transform.baseVal.initialize(transform);

        return svgRoot;
    }

    function rectMoved(annotationGroup: SVGGElement, event: MouseEvent): void {
        setGroupPos(annotationGroup, event.clientX, event.clientY);
    }

    function rectReleased(annotationGroup: SVGGElement, event: MouseEvent): void {
        if (event.button !== 0) {
            // not the left mouse button; ignore
            return;
        }

        const svgRoot = setGroupPos(annotationGroup, event.clientX, event.clientY);
        if (svgRoot === null) {
            return;
        }

        // forget all events
        const removeTheseEvents = dragEvents.splice(0, dragEvents.length);
        for (let removeThisEvent of removeTheseEvents) {
            svgRoot.removeEventListener(removeThisEvent[0], removeThisEvent[1]);
        }

        // forget the drag state
        dragStart = null;
    }

    function rectGrabbed(annotationGroup: SVGGElement, event: MouseEvent): void {
        if (event.button !== 0) {
            // not the left mouse button; ignore
            return;
        }

        // do not pass through to group
        event.stopPropagation();

        const svgRoot = annotationGroup.ownerSVGElement;
        if (svgRoot === null) {
            return;
        }

        // where is the group now?
        const curPos = positionFromTranslate(annotationGroup, SVGLength.SVG_LENGTHTYPE_NUMBER);
        if (curPos === null) {
            return;
        }

        dragStart = {
            x: curPos.x - event.clientX,
            y: curPos.y - event.clientY,
        };

        // set up more events
        const moveEvent = e => rectMoved(annotationGroup, e);
        dragEvents.push(["mousemove", moveEvent]);
        svgRoot.addEventListener("mousemove", moveEvent);

        const releaseEvent = e => rectReleased(annotationGroup, e);
        dragEvents.push(["mouseup", releaseEvent]);
        svgRoot.addEventListener("mouseup", releaseEvent);
    }

    function createDefaultAnnotation(initialText: string): Annotation {
        return {
            left: 0,
            bottom: 0,
            elements: [
                {
                    text: initialText,
                    font_variant: "Regular",
                    font_size: 12,
                    character_spacing: 0,
                    word_spacing: 0,
                    leading: 0,
                    language: null,
                    alternate_text: null,
                    actual_text: null,
                    expansion: null,
                }
            ],
        };
    }

    function makeGroupFromAnnotation(pageGroup: SVGGElement, pageHeightPt: number, annotation: Annotation): SVGGElement|null {
        const svgRoot = pageGroup.ownerSVGElement;
        if (svgRoot === null) {
            return null;
        }

        // convert coordinates to pixels
        const lengther = svgRoot.createSVGLength();

        lengther.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, annotation.left);
        lengther.convertToSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_NUMBER);
        const xPx = lengther.valueInSpecifiedUnits;

        lengther.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, pageHeightPt - annotation.bottom);
        lengther.convertToSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_NUMBER);
        const yPx = lengther.valueInSpecifiedUnits;

        const annoGroup = document.createElementNS(SVG_NS, "g");
        annoGroup.classList.add("annotation");
        const transform = svgRoot.createSVGTransform();
        transform.setTranslate(xPx, yPx);
        annoGroup.transform.baseVal.initialize(transform);

        const grabRect = document.createElementNS(SVG_NS, "rect");
        grabRect.x.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 0);
        grabRect.y.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 0);
        grabRect.width.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 10);
        grabRect.height.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 10);
        grabRect.style.fill = "#fff";
        grabRect.style.fillOpacity = "0.5";
        grabRect.addEventListener("mousedown", event => rectGrabbed(annoGroup, event));
        annoGroup.appendChild(grabRect);

        const annoTextElem = document.createElementNS(SVG_NS, "text");
        annoTextElem.style.fill = "#000";
        annoGroup.appendChild(annoTextElem);

        for (let element of annotation.elements) {
            const lineHeightPt = element.font_size + element.leading;

            const annoTSpanElem = document.createElementNS(SVG_NS, "tspan");
            annoTSpanElem.style.fontSize = `${element.font_size}pt`;
            annoTSpanElem.style.letterSpacing = `${element.character_spacing}pt`;
            annoTSpanElem.style.wordSpacing = `${element.word_spacing}pt`;
            annoTSpanElem.style.lineHeight = `${lineHeightPt}pt`;
            annoTextElem.appendChild(annoTSpanElem);

            if (element.language !== null) {
                annoTSpanElem.setAttribute("data-language", element.language);
            }
            if (element.alternate_text !== null) {
                annoTSpanElem.setAttribute("data-alternate-text", element.alternate_text);
            }
            if (element.actual_text !== null) {
                annoTSpanElem.setAttribute("data-actual-text", element.actual_text);
            }
            if (element.expansion !== null) {
                annoTSpanElem.setAttribute("data-expansion", element.expansion);
            }

            const annoTextNode = document.createTextNode(element.text);
            annoTSpanElem.appendChild(annoTextNode);
        }

        pageGroup.appendChild(annoGroup);

        return annoGroup;
    }

    function newAnnotationFormSubmit(textBox: HTMLInputElement, event: SubmitEvent): void {
        event.preventDefault();
        const annotationText = textBox.value;

        const pageGroup = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (pageGroup === null) {
            return;
        }

        const imageHeightPt = getImageHeightPt(pageGroup);
        if (imageHeightPt === null) {
            return;
        }

        const freshAnnotation = createDefaultAnnotation(annotationText);
        makeGroupFromAnnotation(pageGroup, imageHeightPt, freshAnnotation);
    }

    function hookUpNewAnnotationForm(): void {
        const newAnnotationForm = <HTMLFormElement|null>document.getElementById("pdfmcr-new-annotation-form");
        if (newAnnotationForm === null) {
            return;
        }

        const textBox = <HTMLInputElement|null>newAnnotationForm.querySelector("input[type=text]");
        if (textBox === null) {
            return;
        }

        newAnnotationForm.addEventListener("submit", event => newAnnotationFormSubmit(textBox, event));
    }

    function realizeExistingAnnotations(existingAnnotations: PageAnnotations): void {
        const pageGroup = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (pageGroup === null) {
            return;
        }

        const imageHeightPt = getImageHeightPt(pageGroup);
        if (imageHeightPt === null) {
            return;
        }

        for (let annotation of existingAnnotations.annotations) {
            makeGroupFromAnnotation(pageGroup, imageHeightPt, annotation);
        }
    }

    function doInit(existingAnnotations: PageAnnotations): void {
        realizeExistingAnnotations(existingAnnotations);
        hookUpNewAnnotationForm();
    }

    export function init(existingAnnotations?: PageAnnotations|undefined): void {
        if (existingAnnotations === undefined) {
            existingAnnotations = {
                annotations: [],
                artifacts: [],
            };
        }
        document.addEventListener("DOMContentLoaded", () => doInit(existingAnnotations));
    }
}
