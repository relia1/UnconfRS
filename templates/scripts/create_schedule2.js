document.addEventListener('DOMContentLoaded', function() {
    $(document).ready( function () {
        $('.scheduleTable').DataTable({
            searching: false,
            ordering:  false,
            rowReorder: true,
            colReorder: false,
            paging: false,
            rowReorder: {
                selector: 'tr',
                update: false
            },
        });
    });

    document.getElementById('generate_slots').addEventListener('click', function() {
        const numSlots = document.getElementById('num_of_timeslots').value;
        const container = document.getElementById('timeslots_container');
        const template = document.getElementById('timeslot_template');

        container.innerHTML = '';
        for (let i = 0; i < numSlots; i++) {
            const clone = template.content.cloneNode(true);
            clone.querySelector('.slot-number').textContent = i + 1;
            container.appendChild(clone);
        }
    });
});

async function submitForm(event) {
    event.preventDefault(); // Prevent the default form submission
    const form = document.getElementById('scheduleForm');
    const formData = new FormData(form);

    const data = {
        num_of_timeslots: Number(formData.get('num_of_timeslots')),
        start_time: formData.getAll('start_time[]'),
        end_time: formData.getAll('end_time[]'),
    };

    fetch('api/v1/schedules/add', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
    })
        .then(response => response.json());
}

async function generate(event) {
    event.preventDefault(); // Prevent the default form submission

    const generate_schedule = await fetch('/api/v1/schedules/generate', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
    });
    //.then(response => response.json());
    //.catch(error => console.error('Error:', error));
    const parsed_schedule = await generate_schedule.json(); //JSON.parse(generate_schedule);
}

