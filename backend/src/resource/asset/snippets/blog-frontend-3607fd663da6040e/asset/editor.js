export let editor = null;
export function initEditor() {
    const Editor = toastui.Editor;
    editor = new Editor({
        el: document.querySelector('#editor'),
        previewStyle: 'vertical',
        initialEditType: 'wysiwyg',
        height: '500px',
    });
}
export function getContent() {
    return editor.getMarkdown();
}