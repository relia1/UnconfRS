let numOfTimeslots    = 0;
let numOfRooms        = 0;
let scheduleContainer = null;
let displayControls   = null;
let draggedEvent      = null;

class EventElement {
  constructor(div) {
    this.element = div.element || div;
  }

  updatePosition(top, height) {
    this.element.style.top    = top;
    this.element.style.height = height;
  }

  moveTo(column) {
    column.appendChild(this.element);
  }

  updateAttributes(newData) {
    // Map properties to their corresponding data-attributes
    const attributeMap = {
      startTime:  'data-start-time',
      endTime:    'data-end-time',
      roomId:     'data-room-id',
      timeslotId: 'data-timeslot-id',
      sessionId:  'data-session-id',
      scheduleId: 'data-schedule-id',
    };

    // Only update attributes that are provided in newData
    Object.entries(attributeMap).forEach(([key, attr]) => {
      if (newData[key] !== undefined) {
        this.element.setAttribute(attr, newData[key]);
      }
    });

    if (newData.title !== undefined) {
      this.element.textContent = newData.title;
    }
  }

  getData() {
    return new EventData({...this.element.dataset, title: this.element.textContent});
  }
}

class EventData {
  constructor({roomId, timeslotId, sessionId, scheduleId, startTime, endTime, title}) {
    this.roomId     = Number(roomId);
    this.timeslotId = Number(timeslotId);
    this.sessionId  = Number(sessionId);
    this.scheduleId = Number(scheduleId);
    this.startTime  = startTime + ':00';
    this.endTime    = endTime + ':00';
    this.title      = title;
  }
}

const STATUS_CODES = Object.freeze({
  UNAUTHORIZED: 401,
});

document.addEventListener('DOMContentLoaded', function() {
  scheduleContainer = document.querySelector('.schedule-container');
  displayControls   = document.getElementById('controls');
  let timeslots     = window.APP.timeslots;
  let rooms         = window.APP.rooms;
  let events        = window.APP.events;

  numOfRooms     = rooms.length;
  numOfTimeslots = timeslots.length;

  function createEventBlock(event, displayType) {
    const div     = document.createElement('div');
    div.className = 'event-block';
    div.draggable = true;

    const eventElement = new EventElement(div);
    eventElement.updateAttributes({
      startTime:  event.startTime.substring(0, 5),
      endTime:    event.endTime.substring(0, 5),
      roomId:     event.roomId,
      timeslotId: event.timeslotId,
      title:      event.title,
      sessionId:  event.sessionId,
      scheduleId: event.scheduleId,
    });

    div.addEventListener('dragstart', handleDragStart);

    const {top, height} = displayType === 'time' ?
                          calculateEventPosition(event.startTime.substring(0, 5)) :
                          calculateRoomBasedEventPosition(event.roomId);

    eventElement.updatePosition(top, height);

    return div;
  }

  function calculatePosition(index, total) {
    const multiplier = (1 / (total + 1)) * 100;
    const top        = (((index + 1) * multiplier) + 1) + '%';
    const height     = (multiplier - 1.5) + '%';
    return {top, height};
  }

  function calculateEventPosition(startTime) {
    const startIndex = timeslots.findIndex(slot => slot.start.substring(0, 5) === startTime);

    if (startIndex === -1) {
      return {top: '0%', height: '0%'};
    } else {
      return calculatePosition(startIndex, numOfTimeslots);
    }
  }

  function calculateRoomBasedEventPosition(roomId) {
    return calculatePosition(rooms.findIndex(room => room.id === roomId), numOfRooms);
  }

  function handleDragStart(event) {
    draggedEvent                     = new EventElement(event.target);
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', null);
  }

  function handleDragOver(event) {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }

  async function handleDrop(dropEvent) {
    dropEvent.preventDefault();
    if (!draggedEvent) {
      return;
    }

    const draggedEventData              = draggedEvent.getData();
    const draggedEventElement           = draggedEvent.element;
    const view                          = document.getElementById('view-selector').value;
    const [timeslotIndex, targetRoomId] = await getTimeslotIndexAndRoomId(dropEvent, view);
    if (timeslotIndex === undefined || targetRoomId === undefined) {
      draggedEvent = null;
      return;
    }

    const existingEvent = events.find(event => event.timeslotId ===
        timeslots[timeslotIndex].id &&
        event.roomId === targetRoomId);

    // If there is an event in the same timeslot or room, swap the events
    if (existingEvent) {
      // If the dragged event is dropped on itself, do nothing
      if (existingEvent.timeslotId === draggedEventData.timeslotId && existingEvent.roomId ===
          draggedEventData.roomId) {
        draggedEvent = null;
        return;
      }

      try {
        await swapAssignedEvents(draggedEventData, existingEvent);
      } catch (error) {
        draggedEvent = null;
        return;
      }
    } else {
      const newData = {
        roomId:     targetRoomId,
        timeslotId: timeslots[timeslotIndex].id,
        startTime:  timeslots[timeslotIndex].start.substring(0, 5),
        endTime:    timeslots[timeslotIndex].end.substring(0, 5),
        title:      draggedEventData.title,
        sessionId:  draggedEventData.sessionId,
        scheduleId: draggedEventData.scheduleId,
      };
      try {
        await updateEventInBackend(newData, draggedEventData);
      } catch (error) {
        draggedEvent = null;
        return;
      }
    }

    updateView();
    draggedEvent = null;
  }

  async function getTimeslotIndexAndRoomId(dropEvent, view) {
    const targetColumn = dropEvent.target.closest('.column');
    if (!targetColumn) {
      return [undefined, undefined];
    }

    const rect             = targetColumn.getBoundingClientRect();
    const relativePosition = (dropEvent.clientY - rect.top) / rect.height;

    let [timeslotIndex, roomId] = (() => {
      if (view === 'time') {
        return [
          Math.floor(relativePosition * (numOfTimeslots + 1)) - 1,
          Number(targetColumn.getAttribute('data-room-id')),
        ];
      } else {
        const columnTime = targetColumn.getAttribute('data-time');
        return [
          timeslots.findIndex(slot => slot.start.substring(0, 5) === columnTime),
          rooms[Math.floor(relativePosition * (numOfRooms + 1)) - 1].id,
        ];
      }
    })();

    if (timeslotIndex < 0 || timeslotIndex >= numOfTimeslots) {
      timeslotIndex = undefined;
    }

    if (roomId < 0) {
      roomId = undefined;
    }

    return [timeslotIndex, roomId];
  }

  async function getExistingEventElement(dropEvent, roomId, view, draggedEventData) {
    const targetColumn = dropEvent.target.closest('.column');
    return Array.from(targetColumn.children).find(child => {
      if (!child.classList.contains('event-block')) {
        return false;
      }

      if (view === 'time' || child === draggedEvent.element) {
        return child.getAttribute('data-start-time') === draggedEventData.startTime;
      } else {
        return Number(child.getAttribute('data-room-id')) === roomId;
      }
    });
  }

  async function updateEventInBackend(newData, originalData) {
    const response = await fetch(`api/v1/timeslots/${originalData.timeslotId}`, {
      method:  'PUT',
      headers: {'Content-Type': 'application/json'},
      body:    JSON.stringify({
        id:          newData.timeslotId,
        start_time:  newData.startTime,
        end_time:    newData.endTime,
        session_id:  originalData.sessionId,
        room_id:     Number(newData.roomId),
        old_room_id: originalData.roomId,
        schedule_id: originalData.scheduleId,
      }),
    });

    if (!response.ok) {
      if (response.status === STATUS_CODES.UNAUTHORIZED) {
        alert('You do not have permission to move events');
      } else {
        alert('There was an error updating the schedule. Please try again.');
      }
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    // Update local events array
    const eventIndex = events.findIndex(e => e.timeslotId === originalData.timeslotId &&
        e.roomId === originalData.roomId);
    if (eventIndex !== -1) {
      events[eventIndex] = {...events[eventIndex], ...newData};
    }
  }

  async function swapAssignedEvents(draggedEvent, targetEvent) {
    const response = await fetch(`api/v1/timeslots/swap`, {
      method:  'PUT',
      headers: {'Content-Type': 'application/json'},
      body:    JSON.stringify({
        timeslot_id_1: draggedEvent.timeslotId,
        timeslot_id_2: targetEvent.timeslotId,
        room_id_1:     draggedEvent.roomId,
        room_id_2:     targetEvent.roomId,
      }),
    });

    if (!response.ok) {
      if (response.status === STATUS_CODES.UNAUTHORIZED) {
        alert('You do not have permission to move events');
      } else {
        alert('There was an error updating the schedule. Please try again.');
      }
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    // Update local events array
    const draggedEventIndex = events.findIndex(e => e.timeslotId === draggedEvent.timeslotId &&
        e.roomId === draggedEvent.roomId);
    const targetEventIndex  = events.findIndex(e => e.timeslotId === targetEvent.timeslotId &&
        e.roomId === targetEvent.roomId);
    if (draggedEventIndex !== -1 && targetEventIndex !== -1) {
      const swapToDraggedEvent = {
        ...Object.fromEntries(
            Object.entries(events[targetEventIndex])
                  .filter(([key]) => ['title', 'sessionId'].includes(key)),
        ),
      };
      const swapToTargetEvent  = {
        ...Object.fromEntries(
            Object.entries(events[draggedEventIndex])
                  .filter(([key]) => ['title', 'sessionId'].includes(key)),
        ),
      };

      events[draggedEventIndex] = {...events[draggedEventIndex], ...swapToDraggedEvent};
      events[targetEventIndex]  = {...events[targetEventIndex], ...swapToTargetEvent};
    }
  }

  if (document.getElementById('schedule-operations')) {

    document.getElementById('populate-schedule').addEventListener('click', async () => {
      try {
        const response = await fetch('/api/v1/schedules/generate', {
          method:  'POST',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        location.reload();
      } catch (error) {
        console.error('Error populating schedule:', error);
        alert('There was an error populating the schedule. Please try again.');
      }
    });

    document.getElementById('clear-schedule').addEventListener('click', async () => {
      try {
        const response = await fetch('/api/v1/schedules/clear', {
          method:  'POST',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        location.reload();
      } catch (error) {
        console.error('Error clearing schedule:', error);
        alert('There was an error clearing the schedule. Please try again.');
      }
    });

    document.getElementById('add-room').addEventListener('click', async () => {
      const popup           = document.getElementById('popup');
      const overlay         = document.getElementById('overlay');
      popup.style.display   = 'block';
      overlay.style.display = 'block';
      document.querySelector('#cancelButton').addEventListener('click', closePopup);
      document.querySelector('#overlay').addEventListener('click', closePopup);
    });

    document.getElementById('submit-btn').addEventListener('click', async (e) => {
      e.preventDefault();
      const createRoomsForm = {
        rooms: [
          {
            name:            document.getElementById('room-name').value,
            location:        document.getElementById('room-location').value,
            available_spots: Number(20),
          },
        ],
      };
      try {
        const response = await fetch('/api/v1/rooms/add', {
          method:  'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body:    JSON.stringify(createRoomsForm),
        });
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        closePopup();
        location.reload();
      } catch (error) {
        console.error('Error submitting room:', error);
        alert('There was an error submitting the room. Please try again.');
      }
    });
  }

  function closePopup() {
    const popup           = document.getElementById('popup');
    const overlay         = document.getElementById('overlay');
    popup.style.display   = 'none';
    overlay.style.display = 'none';
    document.querySelector('#cancelButton').removeEventListener('click', closePopup);
    document.querySelector('#overlay').removeEventListener('click', closePopup);
  }

  async function removeRoom(roomId) {
    try {
      const response = await fetch(`/api/v1/rooms/${roomId}`, {
        method:  'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (response.status === STATUS_CODES.UNAUTHORIZED) {
        alert('You do not have permission to delete rooms');
        return;
      }

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      // Remove the room from the rooms array
      const roomIndex = rooms.findIndex(room => room.id === Number(roomId));
      if (roomIndex !== -1) {
        rooms.splice(roomIndex, 1);
      }

      numOfRooms = rooms.length;

      // Reload page
      location.reload();

    } catch (error) {
      console.error('Error deleting room:', error);
      alert('There was an error deleting the room. Please try again.');
    }
  }

  document.addEventListener('click', async function(e) {
    if (e.target && e.target.classList.contains('delete-room-btn')) {
      if (confirm('Are you sure you want to delete this room?')) {
        const roomId = e.target.getAttribute('data-room-id');
        await removeRoom(roomId);
      }
    }
  });

  function updateView() {
    const view = document.getElementById('view-selector').value;
    view === 'time' ? generateTimeBasedView() : generateRoomBasedView();
  }

  function generateTimeBasedView() {
    scheduleContainer.innerHTML = '';

    // Create time slots column
    let rowColumn             = document.createElement('div');
    rowColumn.className       = 'row-column';
    let rowColumnHeader       = document.createElement('div');
    rowColumnHeader.className = 'column-header';
    rowColumnHeader.innerText = 'Time';
    rowColumn.appendChild(rowColumnHeader);

    // Add time labels based on actual timeslots
    timeslots.forEach(slot => {
      const rowLabel       = document.createElement('div');
      rowLabel.className   = 'row-label';
      rowLabel.textContent = slot.start.substring(0, 5);
      rowColumn.appendChild(rowLabel);
    });

    scheduleContainer.appendChild(rowColumn);

    // Create room columns
    rooms.forEach((room, index) => {
      const column     = document.createElement('div');
      column.className = 'column';
      column.setAttribute('data-room-id', room.id);
      column.innerHTML = `
                <div class="column-header">
                    ${room.name}
                    <button class="delete-room-btn" data-room-id="${room.id}">x</button>
                </div>`;
      column.addEventListener('dragover', handleDragOver);
      column.addEventListener('drop', handleDrop);
      scheduleContainer.appendChild(column);

      if (index === 0) {
        column.classList.add('active');
      }
    });

    // Add events to room columns
    events.forEach(event => {
      const column = document.querySelector(`.column[data-room-id="${event.roomId}"]`);
      if (column) {
        const eventBlock = createEventBlock(event, 'time');
        column.appendChild(eventBlock);
      }
    });
  }

  function generateRoomBasedView() {
    scheduleContainer.innerHTML = '';

    // Create rooms column
    let rowColumn             = document.createElement('div');
    rowColumn.className       = 'row-column';
    let rowColumnHeader       = document.createElement('div');
    rowColumnHeader.className = 'column-header';
    rowColumnHeader.innerText = 'Room';
    rowColumn.appendChild(rowColumnHeader);

    // Add room labels
    rooms.forEach(room => {
      const rowLabel       = document.createElement('div');
      rowLabel.className   = 'row-label';
      rowLabel.textContent = room.name;
      rowColumn.appendChild(rowLabel);
    });

    scheduleContainer.appendChild(rowColumn);

    // Create time columns based on actual timeslots
    timeslots.forEach((slot, index) => {
      const column     = document.createElement('div');
      column.className = 'column';
      column.setAttribute('data-time', slot.start.substring(0, 5));
      column.innerHTML = `<div class="column-header">${slot.start.substring(0, 5)}</div>`;
      column.addEventListener('dragover', handleDragOver);
      column.addEventListener('drop', handleDrop);
      scheduleContainer.appendChild(column);

      if (index === 0) {
        column.classList.add('active');
      }
    });

    // Add events to time columns
    events.forEach(event => {
      const column = document.querySelector(`.column[data-time="${event.startTime.substring(
          0,
          5,
      )}"]`);
      if (column) {
        const eventBlock = createEventBlock(event, 'room');
        column.appendChild(eventBlock);
      }
    });
  }

  // Initialize view selector handler and initial view
  document.getElementById('view-selector').addEventListener('change', updateView);
  updateView();
});