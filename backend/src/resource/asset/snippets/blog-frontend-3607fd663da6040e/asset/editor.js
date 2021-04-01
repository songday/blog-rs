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

export function selectTag(tagBox) {
    addTag(tagBox.childNodes[0].nodeValue);
}

function addTag(val) {
    if (!val)
        return;
    initTagElements();
    const tag = document.createElement('span');
    tag.className = "tagBox";

    const tagText = document.createTextNode(val);
    tag.appendChild(tagText);

    const a = document.createElement('a');
    a.innerHTML = 'X';
    a.addEventListener('click', function () {
        allTagsBox.removeChild(tag);
    })
    a.className = 'tagRemove';
    tag.appendChild(a);

    allTagsBox.insertBefore(tag, tagInput);
}

export function getSelectedTags() {
    const tags = [];
    for (let i = 0; i < allTagsBox.childNodes.length; i++) {
        if (allTagsBox.childNodes[i].tagName === 'SPAN')
            tags.push(allTagsBox.childNodes[i].firstChild.nodeValue);
    }
    return tags;
}
