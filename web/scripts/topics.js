document.addEventListener('DOMContentLoaded', function () {
    let table = new DataTable('.topicsTable', {
        columns: [
            {
                data: null,
                className: 'dt-control',
                defaultContent: '',
                orderable: false
            },
            {data: 'topic_id'},
            {data: 'title'},
            {data: 'speaker_id'},
            {data: 'content', visible: false},
        ],
        searching: true,
        ordering: true,
        paging: true,
        responsive: true,
        order: [[1, 'asc']],
    });

    // Add event listener for opening and closing details
    table.on('click', 'td.dt-control', function (e) {
        let tr = e.target.closest('tr');
        let row = table.row(tr);

        if (row.child.isShown()) {
            // This row is already open - close it
            row.child.hide();
        }
        else {
            // Open this row
            row.child(format(row.data())).show();
        }
    });

    table.on('click', 'tbody tr', (e) => {
        let classList = e.currentTarget.classList;

        if (classList.contains('selected')) {
            classList.remove('selected');
        }
        else {
            table.rows('.selected').nodes().each((row) => row.classList.remove('selected'));
            classList.add('selected');
        }
    });

    document.querySelector('#deleteTopicButton').addEventListener('click', async function (data) {


        let topicId = Number(document.querySelector('.selected').dataset.topicId)
        try {
            const response = await fetch(`/api/v1/topics/${topicId}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json'
                },
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            /* Topic was deleted successfully from the database so now also remove it from the
             table */
            table.row('.selected').remove().draw(false);

        } catch (error) {
            console.error('Error deleting topic:', error);
            alert('There was an error deleting the topic. Please try again.');
        }
    });
});

function format(data) {
    // data is the original data object for the row
    return (
        '<dl>' + '<dt><b>Topic Description</b></dt>' + '<dd>' + data.content + '</dd>' + '</dl>'
    );
}

async function submitForm(event) {
    /* Prevent the default form submission */
    event.preventDefault();

    const title = document.getElementById('title').value;
    const content = document.getElementById('topicContent').value;
    const name = document.getElementById('name').value;
    const phone_number = document.getElementById('phone').value;
    const email = document.getElementById('email').value;

    try {
        const add_speaker = await fetch('/api/v1/speakers/add', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({content, name, email, phone_number})
        })
            .then(response => response.json());
        /*.catch(error => console.error('Error:', error));*/
        const speaker_id = await JSON.parse(add_speaker).id;

        try {
            await fetch('/api/v1/topics/add', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({speaker_id, title, content})
            })
                .then(response => response.json())
                .then(() => alert('Topic submitted successfully!'))
                .catch(error => console.error('Error:', error));
        } catch (error) {
            console.log('Error submitting topic: ', error);
        }
        location.reload();
    } catch (error) {
        console.log('Error adding speaker: ', error);
    }
}