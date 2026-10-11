import { defineStore } from 'pinia'

// A toast AND its history entry are the same object, so anything that has ever
// been shown can be read back from the header bell. Kept in localStorage so the
// history survives a reload and an accidental sign-out; capped so it cannot grow
// without bound, and the cap drops the OLDEST entries, never the unread ones.
const HISTORY_KEY = 'lulu:notifications'
const MAX_HISTORY = 30

export const useNotifyStore = defineStore('notify', {
    state: () => ({
        // Currently visible toasts. Each has { id, kind, title, message }.
        toasts: [],
        // Everything ever shown this session and across reloads, newest first.
        history: [],
        nextId: 1,
    }),

    getters: {
        unreadCount: (s) => s.history.filter(n => !n.read).length,
        // Newest first, so the dropdown can render the list without sorting.
        orderedHistory: (s) => [...s.history].sort((a, b) => b.at - a.at),
    },

    actions: {
        // Rehydrate on the client only: during SSR there is no localStorage, and
        // reading it would also make the header badge mismatch between server and
        // client render.
        hydrate() {
            if (!import.meta.client) return
            try {
                const raw = localStorage.getItem(HISTORY_KEY)
                const parsed = raw ? JSON.parse(raw) : []
                if (Array.isArray(parsed)) {
                    this.history = parsed.filter(n => n && typeof n.id === 'number')
                    // Keep the id counter ahead of anything restored, otherwise a
                    // new toast would collide with a reloaded entry.
                    this.nextId = this.history.reduce((m, n) => Math.max(m, n.id), 0) + 1
                }
            } catch {
                // Corrupt or unreadable storage is not worth failing the app over.
                this.history = []
            }
        },

        // kind: 'success' | 'error' | 'info' | 'warning'
        push(kind, title, message = '', ms) {
            // Failures linger: they usually carry text the user has to read and
            // often have to act on, so they outlast the default window.
            if (ms === undefined) ms = (kind === 'error' || kind === 'warning') ? 6000 : 4000
            const note = {
                id: this.nextId++,
                kind,
                title,
                message,
                at: Date.now(),
                read: false,
            }
            this.history.unshift(note)
            if (this.history.length > MAX_HISTORY) {
                this.history.length = MAX_HISTORY
            }
            this.persist()

            this.toasts.push(note)
            // The toast is removed from the stack but stays in history - that is
            // the whole point of the split.
            if (ms > 0) {
                setTimeout(() => this.dismiss(note.id), ms)
            }
            return note.id
        },

        success(title, message = '', ms) { return this.push('success', title, message, ms) },
        error(title, message = '', ms) { return this.push('error', title, message, ms) },
        info(title, message = '', ms) { return this.push('info', title, message, ms) },
        warning(title, message = '', ms) { return this.push('warning', title, message, ms) },

        dismiss(id) {
            const i = this.toasts.findIndex(t => t.id === id)
            if (i !== -1) this.toasts.splice(i, 1)
        },

        markAllRead() {
            for (const n of this.history) n.read = true
            this.persist()
        },

        markRead(id) {
            const n = this.history.find(x => x.id === id)
            if (n) {
                n.read = true
                this.persist()
            }
        },

        clearHistory() {
            this.history = []
            this.persist()
        },

        // Sign-out wipe: drops the visible stack too, so nothing from the previous
        // session flashes on screen during the reload that follows a logout.
        reset() {
            this.history = []
            this.toasts = []
            this.persist()
        },

        persist() {
            try {
                localStorage.setItem(HISTORY_KEY, JSON.stringify(this.history))
            } catch {
                // Private mode or a full quota must never break a page render.
            }
        },
    },
})
