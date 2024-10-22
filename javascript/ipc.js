if (!window.__CROWSER) {
  window.__CROWSER = {}

  /**
   * IPC related functionality, such as events and sending messages
   */
  window.__CROWSER.ipc = {
    // List of messages to send
    // id, cmd, args
    outbound_invokes: [],

    // List of events to listen for
    // id, evt, callback
    inbound_events: {},

    // List of messages to consume
    // id, cmd, args
    inbound_invokes: {},

    // Master message queue to consume from
    // id, type, payload
    message_queue: [],

    invoke: (cmd, args = {}) => {
      if (cmd === "") {
        console.error("[Crowser IPC] Empty command")
        return
      }

      if (typeof args !== "object") {
        console.error("[Crowser IPC] Args must be an object")
        return
      }

      let uuid = generateUUID()
      window.__CROWSER.ipc.outbound_invokes.push({ uuid, cmd, args })

      // Wait for a response
      return new Promise(async (resolve, reject) => {
        while (window.__CROWSER.ipc.inbound_invokes[uuid] === undefined) {
          await wait(5)
        }

        resolve(window.__CROWSER.ipc.inbound_invokes[uuid])
      })
    },
    event: {
      listen: (evt, callback) => {
        if (!window.__CROWSER.ipc.inbound_events[evt]) {
          window.__CROWSER.ipc.inbound_events[evt] = []
        }

        window.__CROWSER.ipc.inbound_events[evt].push(callback)

        return () => {
          window.__CROWSER.ipc.inbound_events[evt] = window.__CROWSER.ipc.inbound_events[evt].filter(cb => cb !== callback)
        }
      },
      unlisten: (evt, callback) => {
        window.__CROWSER.ipc.inbound_events[evt] = window.__CROWSER.ipc.inbound_events[evt].filter(cb => cb !== callback)
      },
      emit: (evt, payload) => {
        window.__CROWSER.ipc.outbound_event.push({ evt, payload })
      },
    },

    _consume: async () => {
      while (true) {
        await wait(5)

        // Check if any items are in the queue
        if (window.__CROWSER.ipc.message_queue.length === 0) {
          break
        }

        const item = window.__CROWSER.ipc.message_queue.shift()

        if (item.type === 'invoke') {
          const callback = window.__CROWSER.ipc.inbound_invokes[item.id]

          if (callback) {
            callback(item.payload)
          }
        } else if (item.type === 'event') {
          const callbacks = window.__CROWSER.ipc.inbound_events[item.evt]

          if (callbacks) {
            callbacks.forEach(cb => cb(item.payload))
          }
        }
      }
    },

    _backend_consume: () => {
      return window.__CROWSER.ipc.outbound_invokes.shift()
    },

    _backend_respond: (uuid, result) => {
      window.__CROWSER.ipc.inbound_invokes[uuid] = result
    }
  }

  window.__CROWSER.ipc._consume()
}

function wait(ms) {
  return new Promise((r) => setTimeout(r, ms))
}

// https://stackoverflow.com/a/8809472/13438741
function generateUUID() {
  var d = new Date().getTime();//Timestamp
  var d2 = ((typeof performance !== 'undefined') && performance.now && (performance.now() * 1000)) || 0;
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    var r = Math.random() * 16;
    if (d > 0) {
      r = (d + r) % 16 | 0;
      d = Math.floor(d / 16);
    } else {
      r = (d2 + r) % 16 | 0;
      d2 = Math.floor(d2 / 16);
    }
    return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
  });
}