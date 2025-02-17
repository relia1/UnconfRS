let currentSpeakerId = null;
let currentTopicId = null;

document.addEventListener('DOMContentLoaded', function () {
    let table = new DataTable('.topicsTable', {
        columns: [
            {
                data: null,
                className: 'dt-control',
                defaultContent: '',
                orderable: false
            },
            {data: 'topic_id', visible: false},
            {data: 'title'},
            {data: 'name'},
            {data: 'email'},
            {data: 'phone_number'},
            {
                data: null,
                defaultContent: '<button class="del-btn btn-action"' +
                    ' id="deleteTopicButton">Delete</button><button class="edit-btn btn-action"' +
                    ' id="editTopicButton">Edit</button><button class="upvote-btn btn-action"' +
                    ' id="upvoteTopicButton">Upvote</button>',
                orderable: false
            },
            {data: 'content', visible: false},
            {data: 'speaker_id', visible: false},
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

    table.on('click', '.del-btn', async function(e) {
        if(confirm('Are you sure you want to delete this topic?')) {
            let row = table.row($(this).closest('tr'));
            let data = row.data();
            console.log("Deleting topic with id: " + data.topic_id);
            try {
                const response = await fetch(`/api/v1/topics/${data.topic_id}`, {
                    method: 'DELETE',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                });

                if(!response.ok) {
                    const error = await response.json();
                    throw new Error(error.error);
                }

                /* Topic was deleted successfully from the database so now also remove
                 it from the table */
                row.remove().draw(false);
            } catch (error) {
                console.error('Error deleting topic:', error);
                if(error.message.match(/foreign key constraint/)) {
                    alert('This topic cannot be deleted because it is associated with a' +
                        ' schedule session.');
                } else {
                    alert('There was an error deleting the topic. Please try again.');
                }
            }
        }
    });

    table.on('click', '.edit-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        currentTopicId = data.topic_id;
        currentSpeakerId = data.speaker_id;
        console.log("Editing topic with id: " + data.topic_id);
        showPopup(true, data);
    });

    table.on('click', '.upvote-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        currentTopicId = data.topic_id;
        currentSpeakerId = data.speaker_id;

        if(!await hasVoted(data.topic_id)) {
            try {
                const response = await fetch(`/api/v1/topics/${data.topic_id}/increment`, {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                })
                    .then(() => alert('Topic upvoted successfully!'))
                    .catch(error => console.error('Error:', error));

                await setVotesCookie(data.topic_id);
            } catch (error) {
                console.error('Error upvoting topic:', error);
                alert('There was an error upvoting the topic. Please try again.');
            }
        } else {
            alert('You have already upvoted this topic!');
        }
    });

    document.getElementById( 'topicForm').addEventListener('submit', async function(event) {
        event.preventDefault();
        const title = document.getElementById('title').value;
        const content = document.getElementById('topicContent').value;
        const name = document.getElementById('name').value;
        const phone_number = document.getElementById('phone').value;
        const email = document.getElementById('email').value;
        const speaker_id = Number(currentSpeakerId);
        const isEdit = currentTopicId !== null;

        let response;
        try {
            if (isEdit) {
                const response = await fetch(`/api/v1/speakers/${currentSpeakerId}`, {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        title,
                        id: Number(currentSpeakerId),
                        content,
                        name,
                        email,
                        phone_number
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                try {
                    await fetch(`/api/v1/topics/${currentTopicId}`, {
                        method: 'PUT',
                        headers: {
                            'Content-Type': 'application/json'
                        },
                        body: JSON.stringify({speaker_id, title, content})
                    })
                        .then(() => alert('Topic updated successfully!'))
                        .catch(error => console.error('Error:', error));
                } catch (error) {
                    console.log('Error updating topic: ', error);
                }
                location.reload();
            } else {
                const add_speaker = await fetch('/api/v1/speakers/add', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({content, name, email, phone_number})
                })
                    .then(response => response.json());
                const speaker_id = await JSON.parse(add_speaker).id;

                try {
                    response = await fetch('/api/v1/topics/add', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json'
                        },
                        body: JSON.stringify({speaker_id, title, content})
                    });

                    if (!response.ok) {
                        throw new Error(`HTTP error! status: ${response.status}`);
                    }
                } catch (error) {
                    console.log('Error submitting topic: ', error);
                }
                location.reload();
            }
        } catch (error) {
            console.log('Error updating speaker: ', error);
        }
    });

    table.on('click', '.upvote-btn', async function(e) {
        var data = table.row($(this).closest('tr')).data();
        console.log("Upvoting topic with id: " + data.topic_id);
    });

    document.querySelector('#add-topic').addEventListener('click', async function (data) {
        await showPopup(false);
    });
});

function format(data) {
    // data is the original data object for the row
    return (
        '<dl>' + '<dt><b>Topic Description</b></dt>' + '<dd>' + data.content + '</dd>' + '</dl>'
    );
}

/* Popup functions */
async function showPopup(isEdit, data=null) {
    const popup = document.getElementById('popup');
    const overlay = document.getElementById('overlay');
    popup.style.display = 'block';
    overlay.style.display = 'block';

    if(isEdit && data) {
        document.getElementById('title').value = data.title;
        document.getElementById('topicContent').value = data.content;
        document.getElementById('name').value = data.name;
        document.getElementById('phone').value = data.phone_number;
        document.getElementById('email').value = data.email;
        currentTopicId = data.topic_id;
        currentSpeakerId = data.speaker_id;
    } else {
        document.getElementById('topicForm').reset();
        currentTopicId = null;
        currentSpeakerId = null;
    }

    document.querySelector('#cancelButton').addEventListener('click', closePopup);
    document.querySelector('#overlay').addEventListener('click', closePopup);
}

function closePopup() {
    const popup = document.getElementById('popup');
    const overlay = document.getElementById('overlay');
    popup.style.display = 'none';
    overlay.style.display = 'none';function closePopup() {
    const popup = document.getElementById('popup');
    const overlay = document.getElementById('overlay');
    popup.style.display = 'none';
    overlay.style.display = 'none';

    document.querySelector('#cancelButton').removeEventListener('click', closePopup);
    document.querySelector('#overlay').removeEventListener('click', closePopup);
}

    document.querySelector('#cancelButton').removeEventListener('click', closePopup);
    document.querySelector('#overlay').removeEventListener('click', closePopup);
}

/* Voting functions */
async function hasVoted(topicId) {
    votesCookie = await cookieStore.get('votes');
    return votesCookie?.value.split(',').includes(String(topicId));
}

async function setVotesCookie(topicId) {
    votesCookie = await cookieStore.get('votes');
    if(!votesCookie) {
        cookieStore.set('votes', String(topicId));
    } else {
        cookieStore.set('votes', votesCookie.value.concat(',' + String(topicId)));
    }
}