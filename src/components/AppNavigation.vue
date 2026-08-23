<script setup>
import { primaryNavigation } from '../config/navigation';
import { useCommands } from '../composables/useCommands';

const commands = useCommands();
</script>

<template>
  <aside class="app-navigation">
    <RouterLink
      class="app-wordmark"
      to="/study"
      aria-label="Twill study"
    >
      Twill
    </RouterLink>

    <nav
      class="primary-navigation"
      aria-label="Primary navigation"
    >
      <UButton
        v-for="item in primaryNavigation"
        :key="item.to"
        :to="item.to"
        :label="item.label"
        :leading-icon="item.icon"
        :aria-label="item.label"
        :aria-keyshortcuts="commands.command( item.commandId ).ariaKeyshortcuts"
        :title="commands.command( item.commandId ).tooltip"
        color="neutral"
        active-color="primary"
        variant="ghost"
        active-variant="soft"
        class="navigation-link"
        exact
        block
      />
    </nav>
  </aside>
</template>
