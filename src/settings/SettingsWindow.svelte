<script lang="ts">
  import GeneralTab from "./GeneralTab.svelte";
  import ModelTab from "./ModelTab.svelte";

  type Tab = "general" | "model" | "permissions";
  let activeTab: Tab = $state("general");

  const tabs: { id: Tab; label: string; icon: string }[] = [
    { id: "general", label: "General", icon: "gear" },
    { id: "model", label: "Model", icon: "cpu" },
    { id: "permissions", label: "Permissions", icon: "shield" },
  ];
</script>

<div class="flex flex-col h-screen bg-[#1e1e1e] text-white">
  <!-- Tab bar -->
  <div class="flex border-b border-white/10 px-4 pt-2">
    {#each tabs as tab}
      <button
        class="px-4 py-2 text-sm font-medium transition-colors relative
          {activeTab === tab.id
          ? 'text-white'
          : 'text-white/50 hover:text-white/70'}"
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
        {#if activeTab === tab.id}
          <div
            class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500 rounded-full"
          ></div>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Tab content -->
  <div class="flex-1 overflow-y-auto p-4">
    {#if activeTab === "general"}
      <GeneralTab />
    {:else if activeTab === "model"}
      <ModelTab />
    {:else if activeTab === "permissions"}
      <div class="text-white/50 text-sm">
        Permissions settings (coming soon)
      </div>
    {/if}
  </div>
</div>
