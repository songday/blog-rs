function fetch_get(t, url, callback) {
    const clazzName = t.className;
    t.disabled = true;
    t.className = clazzName + ' is-loading';
    fetch(url).then(response => response.json())
        .then(data => {
            t.className = clazzName;
            t.disabled = false;
            console.log(data);
            if (data.status === 0) {
                if (url.indexOf('push') > -1) {
                    let d = document.createElement('div');
                    d.innerHTML = 'Push successfully';
                    t.parentNode.appendChild(d);
                } else {
                    if (typeof(callback) === 'function')
                        callback(data);
                    else
                        location.href = backUrl;
                }
            } else {
                showErr(data.error.detail);
            }
        })
        .catch(err => {
            console.log(err);
            showErr(err);
        });
}

function fetch_post(t, url, data, backUrl) {
    const clazzName = t.className;
    t.disabled = true;
    t.className = clazzName + ' is-loading';
    let contentType, body
    if (typeof(data.size) === 'undefined') {
        contentType = 'application/json';
        body = JSON.stringify(data);
    } else {
        contentType = 'application/x-www-form-urlencoded';
        let formBody = [];
        data.forEach(function (value, key, map) {
            formBody.push(key + "=" + encodeURIComponent(value));
        });
        body = formBody.join("&");
    }
    const options = {
        method: 'POST',
        body: body,
        headers: {
            'Content-Type': contentType + ';charset=UTF-8'
        }
    };
    fetch(url, options).then(response => response.json())
        .then(data => {
            t.className = clazzName;
            t.disabled = false;
            console.log(data);
            if (data.status === 0) {
                location.href = backUrl;
            } else {
                showErr(data.error.detail);
            }
        })
        .catch(err => {
            console.log(err);
        });
}

function showErr(err) {
    document.getElementById('errorMessage').innerHTML = err;
    document.getElementById('notification').style.display = 'block';
}

document.addEventListener('DOMContentLoaded', () => {
    (document.querySelectorAll('.notification .delete') || []).forEach(($delete) => {
        const $notification = $delete.parentNode;

        $delete.addEventListener('click', () => {
            $notification.style.display = 'none';
        });
    });
});
