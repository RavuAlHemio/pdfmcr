export namespace Splitter {
    function activateSplitter(splitter: Element, event: Event): void {
        // TODO
    }

    function initContainer(container: HTMLElement): void {
        const splitters = container.getElementsByClassName("spl-splitter");
        for (const splitter of splitters) {
            splitter.addEventListener("mousedown", event => activateSplitter(splitter, event));
        }
    }

    export function init(): void {
        // initialize each splitter container
        const splitterContainers = document.getElementsByClassName("spl-splitter-container");
        for (const splitterContainer of splitterContainers) {
            initContainer(<HTMLElement>splitterContainer);
        }
    }
}
