import io from 'socket.io.client'

const BASE_URL = 'https://www.overleaf.com';

if (process.argv.length < 4) {
  console.log("Usage: node client.ts [SESSION_COOKIE_VALUE] [PROJECT_ID].")
  process.exit(1)
}

const args = process.argv.slice(2);
const session_cookie_value = args[0]
const project_id = args[1]

const socket = io.connect(BASE_URL, {
  'force new connection': true,
  extraHeaders: {
    Cookie: `overleaf_session2=${session_cookie_value}`
  },
  query: new URLSearchParams({ projectId: project_id }).toString(),
  // transports: ['polling']
});

socket.on('connect', () => {
  console.log('Connected to the server');
});

socket.on('message', (data) => {
  console.log('Message received from server:', data);
});

socket.on('disconnect', () => {
  console.log('Disconnected from the server');
});

// Handle connection errors
socket.on('error', (error) => {
  console.error('Error occurred:', error);
});

socket.on('connect_error', (error) => {
  console.error('Connection error:', error);
});

socket.on('connect_timeout', () => {
  console.error('Connection timeout');
});

socket.on('reconnect_failed', () => {
  console.error('Reconnection failed');
});

socket.on("joinProjectResponse", (data) => {
  console.log("JOIN PROJECT RESPONSE!!!")
  console.log(data)
});

// const waitForDisconnect = new Promise((resolve) => {
//   socket.on('disconnect', () => {
//     console.log('Disconnected from the server');
//     resolve("Heee");
//   });
// });
//
// (async () => {
//   try {
//     console.log("Gonna wait...")
//     await waitForDisconnect;
//     console.log('Client has disconnected, continuing with next actions...');
//   } catch (err) {
//     console.error('Error:', err);
//   } finally {
//     process.exit(0);
//   }
// })();
