import { Position, positionFromTranslate, SVG_NS } from "./common";

export namespace Annotations {
    let dragStart: Position|null = null;

    function rectGrabbed(annotationGroup: SVGGElement, event: MouseEvent): void {
        // do not pass through to group
        event.stopPropagation();

        // where is the group now?
        const curPos = positionFromTranslate(annotationGroup);

        dragStart = {
            x: event.clientX,
            y: event.clientY,
        };
    }

    function newAnnotationFormSubmit(textBox: HTMLInputElement, event: SubmitEvent): void {
        event.preventDefault();
        const annotationText = textBox.value;

        const pageGroup = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (pageGroup === null) {
            return;
        }

        const svgRoot = pageGroup.ownerSVGElement;
        if (svgRoot === null) {
            return;
        }

        const annotationGroup = document.createElementNS(SVG_NS, "g");
        annotationGroup.classList.add("annotation");
        const annotationGroupTransform = svgRoot.createSVGTransform();
        annotationGroupTransform.setTranslate(0, 0);
        annotationGroup.transform.baseVal.initialize(annotationGroupTransform);
        pageGroup.appendChild(annotationGroup);

        const grabRect = document.createElementNS(SVG_NS, "rect");
        grabRect.x.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 0);
        grabRect.y.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 0);
        grabRect.width.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 10);
        grabRect.height.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 10);
        grabRect.style.fill = "#fff";
        grabRect.style.fillOpacity = "0.5";
        grabRect.addEventListener("mousedown", event => rectGrabbed(annotationGroup, event));
        annotationGroup.appendChild(grabRect);

        const annotationTextElem = document.createElementNS(SVG_NS, "text");
        annotationTextElem.style.fill = "#000";
        annotationGroup.appendChild(annotationTextElem);

        const annotationTextSpanElem = document.createElementNS(SVG_NS, "tspan");
        annotationTextSpanElem.style.fontSize = "12pt";
        annotationTextSpanElem.style.letterSpacing = "0pt";
        annotationTextSpanElem.style.wordSpacing = "0pt";
        annotationTextSpanElem.style.lineHeight = "12pt";
        annotationTextElem.appendChild(annotationTextSpanElem);

        const annotationTextNode = document.createTextNode(annotationText);
        annotationTextSpanElem.appendChild(annotationTextNode);
    }

    function hookUpNewAnnotationForm(): void {
        const newAnnotationForm = document.getElementById("pdfmcr-new-annotation-form");
        if (newAnnotationForm === null) {
            return;
        }

        const textBox = <HTMLInputElement|null>newAnnotationForm.querySelector("input[type=text]");
        if (textBox === null) {
            return;
        }

        newAnnotationForm.addEventListener("submit", event => newAnnotationFormSubmit(textBox, event));
    }

    function doInit(): void {
        hookUpNewAnnotationForm();
    }

    export function init(): void {
        document.addEventListener("DOMContentLoaded", doInit);
    }
}
