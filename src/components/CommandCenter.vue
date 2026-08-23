<script setup>
import { m } from 'motion-v';
import { computed, ref } from 'vue';

import { COMMAND_IDS } from '../commands/registry';
import {
  useCommandHandler,
  useCommands
} from '../composables/useCommands';

const commands = useCommands();
const openReferenceAfterPalette = ref( false );
const paletteOpen = ref( false );
const paletteSearch = ref( '' );
const referenceOpen = ref( false );

useCommandHandler( COMMAND_IDS.commandPaletteOpen, {
  execute: openPalette
});
useCommandHandler( COMMAND_IDS.commandReferenceOpen, {
  execute: openReference
});

const paletteGroups = computed( () => {
  const availableCommands = commands.list().filter( ( command ) => (
    command.id !== COMMAND_IDS.commandPaletteOpen
    && command.available
    && command.enabled
  ) );

  return groupCommands( availableCommands ).map( ( group ) => ({
    ...group,
    items: group.items.map( ( command ) => ({
      description: command.description,
      icon: command.icon,
      id: command.id,
      kbds: command.shortcutParts,
      label: command.label,
      onSelect: () => selectCommand( command.id )
    }) )
  }) );
});

const referenceGroups = computed( () => groupCommands( commands.list() ) );

function groupCommands( availableCommands ) {
  const groups = new Map();

  for ( const command of availableCommands ) {
    if ( !groups.has( command.group ) ) {
      groups.set( command.group, {
        id: command.group.toLowerCase(),
        label: command.group,
        items: []
      });
    }

    groups.get( command.group ).items.push( command );
  }

  return [ ...groups.values() ];
}

function openPalette() {
  paletteSearch.value = '';
  paletteOpen.value = true;
}

function openReference() {
  referenceOpen.value = true;
}

function selectCommand( commandId ) {
  if ( commandId === COMMAND_IDS.commandReferenceOpen ) {
    openReferenceAfterPalette.value = true;
    paletteOpen.value = false;
    return;
  }

  paletteOpen.value = false;
  commands.execute( commandId );
}

function finishPaletteClose() {
  if ( !openReferenceAfterPalette.value ) {
    return;
  }

  openReferenceAfterPalette.value = false;
  commands.execute( COMMAND_IDS.commandReferenceOpen );
}
</script>

<template>
  <UModal
    v-model:open="paletteOpen"
    title="Commands"
    description="Search available actions."
    class="command-dialog command-dialog--palette"
    :ui="{
      body: 'p-0 sm:p-0',
      overlay: 'command-dialog-overlay'
    }"
    @after:leave="finishPaletteClose"
  >
    <template #body>
      <UCommandPalette
        v-model:search-term="paletteSearch"
        :groups="paletteGroups"
        placeholder="Search commands"
        :fuse="{
          fuseOptions: {
            keys: [ 'label', 'description' ],
            threshold: 0.25,
            useTokenSearch: true
          },
          resultLimit: 20
        }"
        preserve-group-order
        class="command-palette"
      >
        <template #empty="{ searchTerm }">
          <div class="command-palette__empty">
            <UIcon name="i-lucide-search-x" />
            <p>{{ searchTerm ? 'No matching commands.' : 'No commands are available.' }}</p>
          </div>
        </template>
      </UCommandPalette>
    </template>
  </UModal>

  <UModal
    v-model:open="referenceOpen"
    title="Keyboard shortcuts"
    description="Fixed defaults for the current platform."
    class="command-dialog command-dialog--reference"
    :ui="{
      overlay: 'command-dialog-overlay'
    }"
  >
    <template #body>
      <div class="shortcut-reference">
        <m.section
          v-for="( group, index ) in referenceGroups"
          :key="group.id"
          class="shortcut-reference__group"
          :initial="{ opacity: 0, y: 5 }"
          :animate="{ opacity: 1, y: 0 }"
          :transition="{
            duration: 0.18,
            delay: index * 0.025
          }"
        >
          <h3>{{ group.label }}</h3>

          <dl>
            <div
              v-for="command in group.items"
              :key="command.id"
              class="shortcut-reference__row"
            >
              <dt>
                <span class="shortcut-reference__label">
                  <UIcon :name="command.icon" />
                  {{ command.label }}
                </span>

                <span class="shortcut-reference__context">
                  {{ command.context }}
                </span>
              </dt>

              <dd>
                <UKbd
                  :value="command.shortcutLabel"
                  size="md"
                />
              </dd>
            </div>
          </dl>
        </m.section>
      </div>
    </template>
  </UModal>
</template>
