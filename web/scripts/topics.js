document.addEventListener('DOMContentLoaded', function () {
    let table = new DataTable('.topicsTable', {
        searching: true,
        ordering: true,
        paging: true,
        responsive: true
    });
});

async function submitForm(event) {
    /* Prevent the default form submission */
    event.preventDefault();

    const title = document.getElementById('title').value;
    const content = document.getElementById('topicContent').value;
    const name = document.getElementById('name').value;
    const phone_number = document.getElementById('phone_number').value;
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
    } catch (error) {
        console.log('Error adding speaker: ', error);
    }
}