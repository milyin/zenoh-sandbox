<template>
  <div class="home-view">
    <div class="welcome-container">
      <h1>Welcome to Zenoh Sandbox</h1>
      <p class="subtitle">Choose where you want to go:</p>

      <div class="cards">
        <router-link to="/nodes" class="card">
          <div class="card-icon">⚙️</div>
          <h2>Zenoh Nodes</h2>
          <p>Manage Zenoh runtime configurations and instances. Create, start, stop, and monitor local and external Zenoh nodes.</p>
          <ul>
            <li>Configure peer, router, and client modes</li>
            <li>Start and manage runtime instances</li>
            <li>View runtime logs and configurations</li>
            <li>Monitor activity and performance</li>
          </ul>
          <div class="card-action">Go to Nodes →</div>
        </router-link>

        <router-link to="/sessions" class="card">
          <div class="card-icon">🔌</div>
          <h2>Sessions</h2>
          <p>Interact with Zenoh sessions for pub/sub messaging, queries, and data operations.</p>
          <ul>
            <li>Open and manage Zenoh sessions</li>
            <li>Publish and subscribe to topics</li>
            <li>Execute queries and operations</li>
            <li>Monitor session activity</li>
          </ul>
          <div class="card-action">Go to Sessions →</div>
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.home-view {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  width: 100%;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  padding: 2rem;
}

.welcome-container {
  max-width: 1200px;
  width: 100%;
}

h1 {
  font-size: 3rem;
  margin: 0 0 0.5rem 0;
  color: #2c3e50;
  text-align: center;
}

.subtitle {
  font-size: 1.5rem;
  color: #5a6c7d;
  text-align: center;
  margin: 0 0 3rem 0;
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 2rem;
  margin-top: 2rem;
}

.card {
  background: white;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
  text-decoration: none;
  color: inherit;
  display: flex;
  flex-direction: column;
}

.card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 12px rgba(0, 0, 0, 0.15);
}

.card-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.card h2 {
  font-size: 1.75rem;
  margin: 0 0 1rem 0;
  color: #2c3e50;
}

.card p {
  font-size: 1rem;
  color: #5a6c7d;
  line-height: 1.6;
  margin: 0 0 1rem 0;
}

.card ul {
  list-style: none;
  padding: 0;
  margin: 0 0 1.5rem 0;
  flex: 1;
}

.card ul li {
  padding: 0.5rem 0;
  color: #5a6c7d;
  position: relative;
  padding-left: 1.5rem;
}

.card ul li::before {
  content: "✓";
  position: absolute;
  left: 0;
  color: #007bff;
  font-weight: bold;
}

.card-action {
  color: #007bff;
  font-weight: 600;
  font-size: 1.1rem;
  margin-top: auto;
}

.card:hover .card-action {
  color: #0056b3;
}
</style>
