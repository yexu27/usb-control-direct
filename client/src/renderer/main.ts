import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { useConnectionStore } from './stores/connection'
import { useSessionStore } from './stores/session'
import { useBootstrapStore } from './stores/bootstrap'
import { onServiceError } from './services/service-events'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)

const connection = useConnectionStore(pinia)
connection.setupListener()
const session = useSessionStore(pinia)
const bootstrap = useBootstrapStore(pinia)

onServiceError((error) => {
  if (error.kind !== 'unauthenticated') {
    return
  }

  bootstrap.clear()
  session.clearSession()
  void connection.disconnect().catch(() => {})
  void router.push('/login')
})

function handleUserActivity(): void {
  if (session.isLoggedIn) {
    session.resetInactivityTimer()
  }
}

window.addEventListener('pointerdown', handleUserActivity)
window.addEventListener('keydown', handleUserActivity)

app.mount('#app')
