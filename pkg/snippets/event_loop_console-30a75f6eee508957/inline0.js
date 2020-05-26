
    export function display(s) {
        let terminal_element = document.querySelector('#terminal');
        if (terminal_element != null) {
            terminal_element.innerText = s;
        } else {
            console.error("Could not find `#terminal` element in HTML document.");
        }
    }
