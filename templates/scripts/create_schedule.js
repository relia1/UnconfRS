document.addEventListener('DOMContentLoaded', function() {
    let table;

    $(document).ready(function () {
        table = $('.scheduleTable').DataTable({
            searching: false,
            responsive: true,
            ordering: false,
            paging: false,
            fixedHeader: true,
            colReorder: {
                selector: 'td'
            }
        });
    });
});
