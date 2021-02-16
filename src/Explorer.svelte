<script lang="ts">
    import { Treeview } from "smelte";

    import { onMount } from "svelte";

    import API from "./api";

    let rootDirectoriesList = {};

    onMount(async () => {
        const response = await API.listRootDirectories();
        rootDirectoriesList = response;
        console.log(rootDirectoriesList);
    });
</script>

<Treeview
    items={[
        {
            text: "test",
            items: [
                { text: "subtest" },
                { text: "subtest2" },
                { text: "subtest3" },
                {
                    text: "subtest4",
                    items: [
                        { text: "subtest" },
                        { text: "subtest2" },
                        { text: "subtest3" },
                        { text: "subtest4" },
                    ],
                },
            ],
        },
        {
            text: "test2",
            items: [
                { text: "subtest" },
                { text: "subtest2" },
                { text: "subtest3" },
                { text: "subtest4" },
            ],
        },
        {
            text: "test3",
            items: [
                { text: "subtest" },
                { text: "subtest2" },
                { text: "subtest3" },
                { text: "subtest4" },
            ],
        },
    ]}
/>
