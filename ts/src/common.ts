export const SVG_NS = "http://www.w3.org/2000/svg";

export interface Position {
    x: number,
    y: number,
}

export function positionFromTranslate(element: SVGGraphicsElement, svgLengthType: number): Position|null {
    const svgRoot = element.ownerSVGElement;
    if (svgRoot === null) {
        return null;
    }

    const xform = element.transform.baseVal;
    if (xform.length !== 1) {
        return null;
    }
    const xform0 = xform.getItem(0);
    if (xform0.type !== SVGTransform.SVG_TRANSFORM_TRANSLATE) {
        return null;
    }

    // a transformation matrix is:
    // (m11 m21 m31 m41)
    // (m12 m22 m32 m42)
    // (m13 m23 m33 m43)
    // (m14 m24 m34 m44)
    //
    // a 2D translation is:
    // (1 0 0 tx)
    // (0 1 0 ty)
    // (0 0 1 tz)
    // (0 0 0  1)

    const sizer = svgRoot.createSVGLength();

    sizer.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_NUMBER, xform0.matrix.m41);
    sizer.convertToSpecifiedUnits(svgLengthType);
    const x = sizer.valueInSpecifiedUnits;

    sizer.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_NUMBER, xform0.matrix.m42);
    sizer.convertToSpecifiedUnits(svgLengthType);
    const y = sizer.valueInSpecifiedUnits;

    return { x, y };
}
