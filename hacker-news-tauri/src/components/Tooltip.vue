<script setup lang="ts">
interface Props {
  content?: string;
}

const props = defineProps<Props>();
</script>

<template>
  <div class="tooltip" v-if="props.content">
    <slot></slot>
    <span class="tooltip-text">{{ props.content }}</span>
  </div>

  <span v-else>
    <slot></slot>
  </span>
</template>

<style scoped>
.tooltip {
  position: relative;
  display: inline-block;
}

.tooltip .tooltip-text {
  visibility: hidden;
  width: auto;
  background-color: black;
  color: #fff;
  text-align: center;
  border-radius: 6px;
  padding: 10px;

  /* Position the tooltip */
  position: absolute;
  z-index: 1;
  left: 0;
  top: -40px;
  font-size: small;
}

.tooltip:hover .tooltip-text {
  visibility: visible;
}
</style>