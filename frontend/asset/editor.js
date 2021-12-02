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
    for (let i = 0; i < tags.length; i++)
        addTag(tags[i]);
}

function addTag(val) {
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
}

export function getSelectedTags() {
    const tags = [];
    for (let i = 0; i < allTagsBox.childNodes.length; i++) {
        if (allTagsBox.childNodes[i].tagName === 'SPAN')
            tags.push(allTagsBox.childNodes[i].firstChild.nodeValue);
    }
    return tags;
}

export function clearSelectedTags() {
    for (let i = 0; i < allTagsBox.childNodes.length; i++) {
        allTagsBox.removeChild(allTagsBox.childNodes[i]);
    }
}

export function goBack() {
    location.href = '/';
}

export function randomTitleImage(post_id) {
    fetch('/tool/random-title-image/' + post_id)
        .then(response => response.json())
        .then(data => {
            console.log(data);
            if (data.status === 0) {
                document.getElementById('title-image').setAttribute("src", "/"+data.data+"?_rnd="+Math.random());
            }
        })
        .catch(err => {
            console.log(err);
        });
}

export const uploadTitleImage = (postId, files) => {
    const file = files[0];
    // check file type
    if (!['image/jpeg', 'image/png'].includes(file.type)) {
        // document.getElementById('uploaded_image').innerHTML = '<div class="alert alert-danger">Only .jpg and .png image are allowed</div>';
        // document.getElementsByName('sample_image')[0].value = '';
        return;
    }
    // check file size
    if (file.size > 2 * 1024 * 1024) {
        // document.getElementById('uploaded_image').innerHTML = '<div class="alert alert-danger">File must be less than 2 MB</div>';
        // document.getElementsByName('sample_image')[0].value = '';
        return;
    }
    const form_data = new FormData();
    form_data.append('file', file);
    form_data.append('title-image-file-name', file.name);
    fetch("/image/upload-title-image/" + postId, {
        method:"POST",
        body : form_data
    }).then(response => response.json()).then(data => {
        // document.getElementById('uploaded_image').innerHTML = '<div class="alert alert-success">Image Uploaded Successfully</div> <img src="'+responseData.image_source+'" class="img-thumbnail" />';
        // document.getElementsByName('sample_image')[0].value = '';
        console.log(data);
        if (data.status === 0) {
            document.getElementById('title-image').setAttribute("src", "/"+data.data.relative_path+"?_rnd="+Math.random());
        }
    });
}