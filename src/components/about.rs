use leptonic::prelude::*;
use leptos::*;


#[component]
pub fn About(cx: Scope) -> impl IntoView {
    view! { cx,
        <iframe
            width="100%"
            src="https://www.youtube.com/embed/OlD2rcm971U"
            title="YouTube video player"
            frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            allowfullscreen=true
        />
        <Box id="content">
            <p>{ "The Mandelbrot set is a mathematical concept that can be visualized on a Cartesian plane with coordinates ranging from -2 to 2 on both the x and y axes." }</p>
            <br/>
            <p>{ "In our system, each NFT (Non-Fungible Token) represents a specific region within these coordinates, defined by a rectangular shape. We have developed custom software that generates a graphical representation for each NFT." }</p>
            <br/>
            <p>{ "The original NFT, called the Origin NFT, is owned by the project DAO (Decentralized Autonomous Organisation) and covers the entire coordinate range of -2 to 2 on both axes." }</p>
            <br/>
            <p>{ "Every NFT has the ability to create a fixed number (20 in this case) of child NFTs within its own boundaries. This hierarchical structure allows us to trace back each NFT to the Origin NFT." }</p>
            <br/>
            <p>{ "Creating a new NFT requires the use of a cryptocurrency token called FUEL. The number of NFTs that can be minted within a parent NFT is limited." }</p>
            <br/>
            <p>{ "To determine which NFTs get minted, users submit bids, and the owner of the parent NFT selects the winning bids." }</p>
            <br/>
            <p>{ "When an NFT is successfully minted, the FUEL used for minting is distributed among all the parent NFTs, and a portion of it is locked within a newly minted NFT." }</p>
            <br/>
            <p>{ "The owner of a parent NFT can set a minimum amount of FUEL that must be used in a mint bid. All child NFTs created within that parent NFT must adhere to this minimum requirement." }</p>
            <br/>
            <p>{ "Owner of an NFT can burn it, given that it doesn't have any child NFTs in it. When an NFT is burned, FUEL that was locked inside of it is transferred to its owner." }</p>
        </Box>
    }
}
