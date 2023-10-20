<script setup lang="ts">
interface Props {
    content?: string;
    large?: boolean;
}

const props = defineProps<Props>();
</script>

<template>
    <div class="tooltip" v-if="props.content">
        <slot></slot>
        <span :class="{ tooltipText: true, large }">{{ props.content }}</span>
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

.tooltip:hover .tooltipText {
    visibility: visible;
}

.tooltip .tooltipText {
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
    top: -45px;
    font-size: small;
}

.tooltip .large {
    width: 130px;
}
</style>
