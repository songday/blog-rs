export let editor = null;

export function initEditor() {
    const Editor = toastui.Editor;
    editor = new Editor({
        el: document.querySelector('#editor'),
        previewStyle: 'vertical',
        initialEditType: 'wysiwyg',
        initialValue: '',
        height: '500px',
    });
}

export function setInitContent(intentContent) {
    if (editor == null)
        this.initEditor();
    editor.setMarkdown(intentContent, false);
}

export function getContent() {
    return editor.getMarkdown();
}

// tag
export let tagInput;
export let allTagsBox;

function initTagElements() {
    if (tagInput && allTagsBox)
        return;
    tagInput = document.getElementById('tagInput');
    tagInput.addEventListener('keyup', inputTag, false);
    allTagsBox = document.getElementById('tagsBox');
}

export function inputTag(event) {
    if (event.keyCode !== 13)
        return;
    initTagElements();
    addTag(tagInput.value);
    tagInput.value = '';
    tagInput.focus();
}

export function selectTag(tag) {
    addTag(tag);
}

export function selectTags(tags) {
    console.log(tags.length);
    for (let i = 0; i < tags.length; i++)
        addTag(tags[i]);
}

function addTag(val) {
    console.log(val);
    if (!val)
        return;
    initTagElements();
    const tag = document.createElement('span');
    tag.className = "me-3 border-end border-3";

    const tagText = document.createTextNode(val);
    tag.appendChild(tagText);

    const btn = document.createElement('i');
    btn.addEventListener('click', function () {
        allTagsBox.removeChild(tag);
    })
    btn.className = 'bi bi-trash';
    tag.appendChild(btn);

    // allTagsBox.insertBefore(tag, tagInput);
    allTagsBox.appendChild(tag);
    console.log("addTag");
}

export function getSelectedTags() {
    const tags = [];
    for (let i = 0; i < allTagsBox.childNodes.length; i++) {
        if (allTagsBox.childNodes[i].tagName === 'SPAN')
            tags.push(allTagsBox.childNodes[i].firstChild.nodeValue);
    }
    return tags;
}

export function gotoLogin() {
    location.href = '/management';
}