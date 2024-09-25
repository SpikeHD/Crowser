if (!window.__CROWSER) {
  window.__CROWSER = {}

  /**
   * IPC related functionality, such as events and sending messages
   */
  window.__CROWSER.ipc = {
    // List of messages to send
    // id, cmd, args
    outbound_invokes: {},

    // List of events to send
    // id, evt, payload
    outbound_event: {},

    // List of events to listen for
    // id, evt, callback
    inbound_events: {},

    // List of messages to consume
    // id, cmd, args
    inbound_invokes: {},

    // Master message queue to consume from
    // id, type, payload
    message_queue: [],

    invoke: (cmd, args) => {
      window.__CROWSER.ipc.outbound_invokes.push({ cmd, args })
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
    }
  }

  window.__CROWSER.ipc._consume()
}

function wait(ms) {
  return new Promise((r) => setTimeout(r, ms))
}