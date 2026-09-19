<script setup>
import { COMMAND_IDS } from '../commands/registry';
import { primaryNavigation } from '../config/navigation';
import { useCommands } from '../composables/useCommands';

defineProps({
  drawer: Boolean
});

defineEmits([ 'navigate' ]);

const commands = useCommands();
const paletteCommand = commands.command( COMMAND_IDS.commandPaletteOpen );
const referenceCommand = commands.command( COMMAND_IDS.commandReferenceOpen );
</script>

<template>
  <aside
    class="app-navigation"
    :class="{ 'app-navigation--drawer': drawer }"
    data-twill-navigation
  >
    <RouterLink
      v-if="!drawer"
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
        :leading-icon="item.icon"
        :aria-label="item.label"
        :aria-keyshortcuts="commands.command( item.commandId ).ariaKeyshortcuts"
        :title="commands.command( item.commandId ).tooltip"
        color="neutral"
        active-color="primary"
        variant="ghost"
        active-variant="subtle"
        class="navigation-link"
        data-twill-navigation-item
        :data-twill-destination="item.to.slice( 1 )"
        exact
        block
        @click="$emit( 'navigate' )"
      >
        <span class="navigation-link__label">{{ item.label }}</span>
      </UButton>
    </nav>

    <div v-if="!drawer" class="command-center-navigation">
      <UButton
        :aria-label="paletteCommand.label"
        :aria-keyshortcuts="paletteCommand.ariaKeyshortcuts"
        :title="paletteCommand.tooltip"
        leading-icon="i-lucide-search"
        color="neutral"
        variant="ghost"
        class="command-entry"
        block
        @click="commands.execute( COMMAND_IDS.commandPaletteOpen )"
      >
        <span class="command-entry__label">Commands</span>
      </UButton>

      <UButton
        :aria-label="referenceCommand.label"
        :aria-keyshortcuts="referenceCommand.ariaKeyshortcuts"
        :title="referenceCommand.tooltip"
        leading-icon="i-lucide-keyboard"
        color="neutral"
        variant="ghost"
        class="command-entry"
        block
        @click="commands.execute( COMMAND_IDS.commandReferenceOpen )"
      >
        <span class="command-entry__label">Shortcuts</span>
      </UButton>
    </div>
  </aside>
</template>
