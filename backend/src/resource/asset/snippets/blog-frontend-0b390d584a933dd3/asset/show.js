export function showNotificationBox() {
    document.getElementById('notification').style.display = 'block';
}
export function hideNotificationBox(event) {
    let source = event.target || event.srcElement;
    while (source.id !== 'notification' && source.parentNode)
        source = source.parentNode;
    source.style.display = 'none';
}