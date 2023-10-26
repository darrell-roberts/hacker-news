<script setup lang="ts">
import Article from "./components/Article.vue";
import { TopStories, Item } from "./types/article";
import { onMounted, onUnmounted, reactive, ref, watch } from "vue";
import Tooltip from "./components/Tooltip.vue";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";

interface State {
    topStories: TopStories;
    error?: string;
    filtered: boolean;
    url?: string;
    unlisten?: UnlistenFn | void | string;
    liveEvents: boolean;
}

const state = reactive<State>({
    topStories: { items: [], totalStories: 0 },
    filtered: false,
    liveEvents: true,
});

onMounted(() => {
    listen<TopStories>("top_stories", (topStories) => {
        mergeViewed(topStories.payload);
    })
        .catch(
            (err) => (state.error = `Failed to listen to top stories ${err}`),
        )
        .then((unlisten) => (state.unlisten = unlisten));
});

onUnmounted(() => {
    if (typeof state.unlisten) {
        (state.unlisten as UnlistenFn)();
    }
});

function toggleFilter() {
    state.filtered = !state.filtered;
}

async function toggleLiveEvents() {
    state.liveEvents = await invoke("toggle_live_events");
}

function applyFilter(item: Item) {
    if (state.filtered) {
        return item.hasRust;
    } else {
        return true;
    }
}

function mergeViewed(items: TopStories) {
    const viewed = state.topStories.items
        .filter((item) => item.viewed)
        .map((item) => item.id);

    for (const item of items.items) {
        if (viewed.includes(item.id)) {
            item.viewed = true;
        }
    }
    state.topStories = items;
}

function onMenu(e: PointerEvent) {
    // No context menu for now.
    e.preventDefault();
}

const selectTotalArticles = ref(0);

const options = ref([
    { text: "10", value: 10 },
    { text: "25", value: 25 },
    { text: "50", value: 50 },
    { text: "75", value: 75 }
]);

watch(state, (state) => {
    if (state.topStories.totalStories !== selectTotalArticles.value) {
        selectTotalArticles.value = state.topStories.totalStories;
    }
});

watch(selectTotalArticles, (change) => {
    invoke("update_total_articles", { totalArticles: change})
        .catch(err => console.error("Failed to update total articles", err));
});
</script>

<template>
    <div class="container" :oncontextmenu="onMenu">
        <div v-if="state.error">Failed to load stories: {{ state.error }}</div>

        <div class="articles">
            <div v-for="(item, index) in state.topStories.items" :key="item.id">
                <Article :item="item" :index="index" v-if="applyFilter(item)" @viewed="() => (item.viewed = true)"
                    @url="(url) => (state.url = url)" />
            </div>
        </div>

        <div class="footer">
            <div class="status-line">
                <Tooltip :content="state.liveEvents
                    ? 'Pause live events'
                    : 'Resume live events'
                    " :large="true">
                    <div>
                        <div v-if="state.topStories.items.length === 0">
                            Loading...
                        </div>
                        <div v-else class="status-action" @click="toggleLiveEvents()">
                            <span>
                                {{ state.topStories.loaded }}
                            </span>
                            <span v-if="state.liveEvents">⚡️</span>
                        </div>
                    </div>
                </Tooltip>
                <div :style="{ display: 'flex' }">
                    <div>
                        <span>Show: </span>
                        <select v-model="selectTotalArticles"
                            :disabled="state.topStories.items.length === 0">
                            <option v-for="option in options" :value="option.value">
                                {{ option.text }}
                            </option>
                        </select>
                    </div>
                    <div>
                        Jobs: {{ state.topStories.items.filter(item => item.ty === "job").length }}
                    </div>
                    <div :style="{ marginLeft: '10px' }">
                        Stories: {{ state.topStories.items.filter(item => item.ty === "story").length }}
                    </div>
                    <div :style="{ marginLeft: '10px' }">
                        Polls: {{ state.topStories.items.filter(item => item.ty === "poll").length }}
                    </div>
                    <Tooltip content="Filter Rust">
                        <div @click="toggleFilter()" class="status-action">
                            Rust articles: {{ state.topStories.rustArticles ?? 0 }}
                        </div>
                    </Tooltip>
                </div>
            </div>

            <div class="url">{{ state.url }}</div>
        </div>
    </div>
</template>

<style scoped>
.articles {
    overflow: auto;
    column-count: 2;
    column-width: 200px;
    column-gap: 2px;
    padding: 5px;
    min-height: 95vh;
}

@media only screen and (max-width: 850px) {
    .articles {
        column-count: 1;
    }
}

@media only screen and (min-width: 1700px) {
    .articles {
        column-count: 3;
    }
}

.status-line {
    color: gray;
    font-size: small;
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    margin-right: 5px;
    margin-top: 5px;
    margin-bottom: 5px;
    line-height: 1.5rem;
}

.footer {
    position: sticky;
    bottom: 0;
    background-color: #2f2f2f;
    padding-right: 5px;
}

.status-action {
    cursor: pointer;
    margin-left: 10px;
}

.status-action:hover {
    color: white;
}

.url {
    text-align: left;
    color: white;
    text-overflow: ellipsis;
    overflow: hidden;
    text-align: left;
    text-wrap: nowrap;
    height: 25px;
    width: 95vw;
    margin-left: 5px;
    white-space: nowrap;
}
</style>
