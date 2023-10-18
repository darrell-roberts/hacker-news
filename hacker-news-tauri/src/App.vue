<script setup lang="ts">
import Article from "./components/Article.vue";
import { TopStories, Item } from "./types/article";
import { onMounted, onUnmounted, reactive } from "vue";
import Tooltip from "./components/Tooltip.vue";
import { UnlistenFn, listen } from '@tauri-apps/api/event'
import { invoke } from "@tauri-apps/api";

interface State {
  topStories: TopStories,
  fetching: boolean,
  error?: string,
  filtered: boolean,
  url?: string
  unlisten?: UnlistenFn | void | string ,
  liveEvents: boolean,
}

const state = reactive<State>({
    topStories: { items: [] },
    fetching: false,
    filtered: false,
    liveEvents: true,
});

onMounted(() => {
    listen<TopStories>("top_stories", (topStories) => {
        mergeViewed(topStories.payload);
    }).catch(err => state.error = `Failed to listen to top stories ${err}`)
    .then(unlisten => state.unlisten = unlisten);
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
    const viewed = state.topStories.items.filter(item => item.viewed).map(item => item.id);

    for (const item of items.items) {
        if (viewed.includes(item.id)) {
            item.viewed = true;
        }
    }
    state.topStories = items;
}
</script>

<template>
  <div class="container">
    <div v-if="state.error">Failed to load stories: {{ state.error }}</div>


    <div class="articles">
        <div v-for="(item, index) in state.topStories.items" :key="item.id">
            <Article
                :item="item"
                :index="index"
                v-if="applyFilter(item)"
                @viewed="() => item.viewed = true"
                @url="(url) => state.url = url"
            />
        </div>
    </div>

    <div class="footer">
        <div class="status-line">
                <Tooltip
                    :content="state.liveEvents ? 'Disable Live Events' : 'Enable Live Events'"
                    :large="true">
                <div>
                    <div v-if="state.fetching">Loading...</div>
                    <div v-else class="status-action" @click="toggleLiveEvents()">
                        <span>
                            {{ state.topStories.loaded }}
                        </span>
                        <span v-if="state.liveEvents">⚡️</span>
                    </div>
                </div>
                </Tooltip>
                <Tooltip content="Filter Rust">
                    <div @click="toggleFilter()" class="status-action">
                        Rust articles: {{ state.topStories.rustArticles }}
                    </div>
                </Tooltip>

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
  padding: 10px;
  min-height: 95vh;
}

@media only screen and (max-width: 850px) {
  .articles {
    column-count: 1;
  }
}

@media only screen and (min-width: 2000px) {
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
  margin-left: 5px;
  margin-right: 5px;
}

.footer {
    position: sticky;
    bottom: 0;
    background-color: #2f2f2f;
    border-radius: 8px;
}

.status-action {
  cursor: pointer;

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
