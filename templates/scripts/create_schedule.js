document.addEventListener('DOMContentLoaded', function() {
    let table;

    $(document).ready(function () {
        table = $('.scheduleTable').DataTable({
            searching: false,
            responsive: true,
            ordering: false,
            paging: false,
            rowReorder: {
                selector: 'tr',
                update: false
            }
        });


        table.on('row-reorder', function (e, diff, edit) {
            for (let i = 0; i < diff.length - 1; i++) {
                let oldRowIndex = diff[i].oldPosition;
                let newRowIndex = diff[i].newPosition;

                // Get the rows involved in the swap
                let oldRow = table.row(oldRowIndex).data();
                let newRow = table.row(newRowIndex).data();

                // Swap only the speaker and topic data
                //let tempTimeslotId = Number(document.querySelectorAll("tbody tr")[oldRowIndex].dataset.timeslotId);
                let tempSpeaker = oldRow[2]; // Assuming speaker is at column index 2
                let tempTopic = oldRow[3]; // Assuming topic is at column index 3
                //document.querySelectorAll("tbody tr")[oldRowIndex].dataset.timeslotId = Number(document.querySelectorAll("tbody tr")[newRowIndex].dataset.timeslotId);
                oldRow[2] = newRow[2];
                oldRow[3] = newRow[3];
                //document.querySelectorAll("tbody tr")[newRowIndex].dataset.timeslotId = tempTimeslotId;
                newRow[2] = tempSpeaker;
                newRow[3] = tempTopic;

                // Update the rows with the new data
                table.row(oldRowIndex).data(oldRow);
                table.row(newRowIndex).data(newRow);
            }

            // Redraw the table to reflect the changes
            table.draw(false);
        });
    });
});

async function createScheduleTemplate() {
    const schedule_timings = {
        time_stepping: [0, 30],
        am_hours: [8, 9, 10, 11],
        pm_hours: [12, 1, 2, 3, 4, 5, 6],
    };
    let am_index = 0;

    const container = document.getElementById('timeslots_container');
    const template = document.getElementById('timeslot_template');
    container.innerHTML = '';

    function createTimeSlot(slotNumber, startTime, endTime) {
        const clone = template.content.cloneNode(true);
        clone.querySelector('.slot-number').textContent = slotNumber;

        const startInput = clone.querySelector('input[name="start_time[]"]');
        const endInput = clone.querySelector('input[name="end_time[]"]');

        startInput.value = startTime;
        endInput.value = endTime;

        container.appendChild(clone);

        // Initialize Flatpickr for both inputs
        flatpickr(startInput, {
            enableTime: true,
            noCalendar: true,
            dateFormat: "H:i",
            time_24hr: true,
            minuteIncrement: 30
        });

        flatpickr(endInput, {
            enableTime: true,
            noCalendar: true,
            dateFormat: "H:i",
            time_24hr: true,
            minuteIncrement: 30
        });
    }

    // Create AM hour timeslots
    schedule_timings.am_hours.forEach((hour, index) => {
        schedule_timings.time_stepping.forEach((minute, i) => {
            const slotNumber = index * 4 + i + 1;
            const startTime = `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
            const endTime = minute === 45 ?
                `${(hour + 1).toString().padStart(2, '0')}:00` :
                `${hour.toString().padStart(2, '0')}:${(minute + 30).toString().padStart(2, '0')}`;
            createTimeSlot(slotNumber, startTime, endTime);
            am_index++;
        });
    });

    // Create PM hour timeslots
    schedule_timings.pm_hours.forEach((hour, index) => {
        schedule_timings.time_stepping.forEach((minute, i) => {
            const slotNumber = am_index + index * 4 + i + 1;
            const adjustedHour = hour === 12 ? 12 : hour + 12;
            const startTime = `${adjustedHour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
            const endTime = minute === 45 ?
                `${(adjustedHour + 1).toString().padStart(2, '0')}:00` :
                `${adjustedHour.toString().padStart(2, '0')}:${(minute + 30).toString().padStart(2, '0')}`;
            createTimeSlot(slotNumber, startTime, endTime);
        });
    });
}

async function submitForm(event) {
    event.preventDefault();
    const form = document.getElementById('scheduleForm');
    const formData = new FormData(form);
    const start_time = formData.getAll('start_time[]');
    const end_time = formData.getAll('end_time[]');

    const data = {
        num_of_timeslots: Number(start_time.length),
        start_time: start_time,
        end_time: end_time,
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
    event.preventDefault();
    const generate_schedule = await fetch('/api/v1/schedules/generate', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
    });
    const parsed_schedule = await generate_schedule.json();
}

async function updateSchedule(event, scheduleId) {
    event.preventDefault();

    const rows = document.querySelectorAll("tbody tr");
    let timeslots = [];
    for (let row of rows) {
        const timeslotId = row.getAttribute('data-timeslot-id');
        const cells = row.cells;

        const startTime = cells[0].textContent.trim();
        const endTime = cells[1].textContent.trim();
        const speakerId = cells[2].textContent.trim();
        const topicId = cells[3].textContent.trim();

        console.log(`Timeslot ID: ${timeslotId}`);
        console.log(`Start Time: ${startTime}`);
        console.log(`End Time: ${endTime}`);
        console.log(`Speaker: ${speakerId}`);
        console.log(`Topic: ${topicId}`);
        console.log('---');

        let timeslot = {
            id: Number(timeslotId),
            start_time: startTime,
            end_time: endTime,
            speaker_id: speakerId === "" ? null : Number(speakerId),
            topic_id: topicId === "" ? null : Number(topicId),
            schedule_id: Number(scheduleId)
        };

        timeslots.push(timeslot);
    }



    let ret = {
        id: scheduleId,
        num_of_timeslots: rows.length,
        timeslots: timeslots
    };

    const generate_schedule = await fetch(`/api/v1/schedules/${scheduleId}`, {
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(ret/*timeslots*/),
    });
    //const parsed_schedule = await generate_schedule.json();
}
// window.onload = createScheduleTemplate;
