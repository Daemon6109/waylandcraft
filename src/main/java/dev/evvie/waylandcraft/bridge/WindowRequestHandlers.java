package dev.evvie.waylandcraft.bridge;

public class WindowRequestHandlers {
	
	@FunctionalInterface
	public static interface MaximizeRequestHandler {
		
		void onMaximizeRequest(WLCToplevel toplevel);
		
	}
	
	@FunctionalInterface
	public static interface UnmaximizeRequestHandler {
		
		void onUnmaximizeRequest(WLCToplevel toplevel);
		
	}
	
	@FunctionalInterface
	public static interface FullscreenRequestHandler {
		
		void onFullscreenRequest(WLCToplevel toplevel);
		
	}
	
	@FunctionalInterface
	public static interface UnfullscreenRequestHandler {
		
		void onUnfullscreenRequest(WLCToplevel toplevel);
		
	}
	
	@FunctionalInterface
	public static interface MinimizeRequestHandler {
		
		void onMinimizeRequest(WLCToplevel toplevel);
		
	}
	
	@FunctionalInterface
	public static interface MoveRequestHandler {
		
		void onMoveRequest(WLCToplevel toplevel, int serial);
		
	}
	
	@FunctionalInterface
	public static interface ResizeRequestHandler {
		
		void onResizeRequest(WLCToplevel toplevel, int serial, int edge);
		
	}
	
	@FunctionalInterface
	public static interface DNDRequestHandler {
		
		void onDNDRequest(WLCToplevel toplevel, int serial);
		
	}
	
}
