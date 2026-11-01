package rev

import "testing"

func TestReverse(t *testing.T) {
	if got := Reverse("héllo"); got != "olléh" {
		t.Fatalf("got %q", got)
	}
}
