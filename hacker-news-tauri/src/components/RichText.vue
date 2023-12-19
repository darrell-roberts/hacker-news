<script setup lang="ts">
import { RichText } from "../types/article";
import { shell } from "@tauri-apps/api";

interface Props {
    richText: RichText[];
}

const props = defineProps<Props>();

function openLink(url: string) {
    shell.open(url);
}
</script>

<template>
    <span>
        <template v-for="richText in props.richText" class="richText">
            <template v-if="richText.type === 'text'">{{
                richText.content
            }}</template>
            <template v-else-if="richText.type === 'char'">{{
                richText.content
            }}</template>
            <p v-else-if="richText.type === 'paragraph'" />
            <span
                class="link"
                v-else-if="richText.type === 'link'"
                @click="() => openLink(richText.content?.link_ref)"
                >{{ richText.content?.name }}</span
            >
            <b v-else-if="richText.type === 'bold'">{{ richText.content }}</b>
            <i v-else-if="richText.type === 'italic'">
                {{ richText.content }}
            </i>
            <pre
                v-else-if="richText.type === 'code'"
            ><code>{{richText.content}}</code></pre>
        </template>
    </span>
</template>

<style scoped>
.link {
    color: blue;
    text-decoration: underline;
    cursor: pointer;
}
</style>
