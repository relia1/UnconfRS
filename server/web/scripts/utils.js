function setButtonLoading(button) {
    const $button = $(button);
    // Store original classes of the button
    $button.data('original-classes', $button.attr('class'));

    // Make it so the button is disabled and add the spinner
    $button.prop('disabled', true);
    $button.html(`${$button.text()}<span class="spinner-border spinner-border-sm me-1" role="status" aria-hidden="true"></span>`);
}

function restoreButton(button, newText = null, newClasses = null) {
    const $button = $(button);
    const text = newText ?? $button.text();
    const classes = newClasses ?? $button.data('original-classes');

    $button.prop('disabled', false);
    $button.html(text);

    if (classes) {
        $button.attr('class', classes);
    }
}
